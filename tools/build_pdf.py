#!/usr/bin/env python3
"""Build the master reader edition and check/render every page, not the runtime."""
from html import escape
import json
from pathlib import Path
import re
import tempfile

from markdown_it import MarkdownIt
import pymupdf
from reportlab.lib import colors
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (BaseDocTemplate, Frame, PageTemplate, Paragraph,
                               Spacer, PageBreak, Preformatted, LongTable, TableStyle)
from reportlab.platypus.tableofcontents import TableOfContents

ROOT = Path(__file__).resolve().parents[1]
WIDTH, HEIGHT = A4
MARGIN = 46
SOURCES = ['MASTER_SPEC.md', 'docs/QUALIFICATION_CONTRACT.md',
           'docs/CONTEXT_ASSEMBLY_CAMPAIGN.md', 'docs/RETRIEVAL_AND_MEMORY.md', 'docs/SELF_IMPROVEMENT.md']

def register_fonts():
    regular = Path('/usr/share/fonts/liberation/LiberationSans-Regular.ttf')
    if not regular.is_file():
        raise RuntimeError('Install/provide Liberation fonts for this reproducible PDF build.')
    for name, filename in [('Body', 'LiberationSans-Regular.ttf'),
                           ('Body-Bold', 'LiberationSans-Bold.ttf'),
                           ('Body-Italic', 'LiberationSans-Italic.ttf'),
                           ('Body-BoldItalic', 'LiberationSans-BoldItalic.ttf'),
                           ('Mono', 'LiberationMono-Regular.ttf')]:
        pdfmetrics.registerFont(TTFont(name, str(regular.parent / filename)))
    pdfmetrics.registerFontFamily('Body', normal='Body', bold='Body-Bold',
                                 italic='Body-Italic', boldItalic='Body-BoldItalic')

def inline(tokens):
    result = []
    for token in tokens or []:
        if token.type == 'text':
            # Bare URLs in the source register remain clickable in the PDF.
            parts = re.split(r'(https?://[^\s]+)', token.content)
            for part in parts:
                encoded = escape(part)
                result.append(f'<link href="{encoded}" color="#155D88">{encoded}</link>'
                              if part.startswith(('https://', 'http://')) else encoded)
        elif token.type == 'code_inline':
            result.append('<font name="Mono" size="8">' + escape(token.content) + '</font>')
        elif token.type in {'strong_open', 'strong_close'}:
            result.append('<b>' if token.type.endswith('open') else '</b>')
        elif token.type in {'em_open', 'em_close'}:
            result.append('<i>' if token.type.endswith('open') else '</i>')
        elif token.type == 'link_open':
            href = token.attrGet('href') or ''
            result.append('<link href="' + escape(href) + '" color="#155D88">'
                          if href.startswith(('http://', 'https://')) else '<font color="#155D88">')
        elif token.type == 'link_close':
            # Pair with the most recent unmatched opener in this simple stream.
            open_link = next((s for s in reversed(result) if s.startswith(('<link ', '<font color='))), '')
            result.append('</link>' if open_link.startswith('<link ') else '</font>')
        elif token.type == 'softbreak':
            result.append(' ')
        elif token.type == 'hardbreak':
            result.append('<br/>')
    return ''.join(result)

class ReaderDoc(BaseDocTemplate):
    def __init__(self, path):
        super().__init__(str(path), pagesize=A4, leftMargin=MARGIN, rightMargin=MARGIN,
                         topMargin=MARGIN, bottomMargin=MARGIN,
                         title='Hephaestus — Autonomous Invention Harness v1.2', author='Hephaestus specification package')
        self.addPageTemplates(PageTemplate(id='body', frames=[Frame(MARGIN, MARGIN,
            WIDTH - 2 * MARGIN, HEIGHT - 2 * MARGIN, leftPadding=0, rightPadding=0,
            topPadding=0, bottomPadding=0)], onPage=self.footer))

    def footer(self, canvas, doc):
        canvas.setFont('Body', 8)
        canvas.setFillColor(colors.HexColor('#526170'))
        canvas.drawString(MARGIN, 25, 'Hephaestus v1.2 · Specification/reference package · 30 September 2026')
        canvas.drawRightString(WIDTH - MARGIN, 25, str(doc.page))

    def afterFlowable(self, flowable):
        if hasattr(flowable, 'bookmark_key'):
            label = flowable.getPlainText()
            self.canv.bookmarkPage(flowable.bookmark_key)
            self.canv.addOutlineEntry(label, flowable.bookmark_key, level=0)
            self.notify('TOCEntry', (0, label, self.page, flowable.bookmark_key))

def build():
    register_fonts()
    styles = getSampleStyleSheet()
    body = ParagraphStyle('BodyTextCustom', fontName='Body', fontSize=9.2, leading=13.2,
                          spaceAfter=7, splitLongWords=True)
    heading = ParagraphStyle('Chapter', parent=body, fontName='Body-Bold', fontSize=16,
                             leading=20, spaceBefore=16, spaceAfter=10, keepWithNext=True)
    subheading = ParagraphStyle('Subheading', parent=heading, fontSize=12, leading=16)
    cell = ParagraphStyle('Cell', parent=body, fontSize=8, leading=11, spaceAfter=0)
    code = ParagraphStyle('Code', parent=body, fontName='Mono', fontSize=7, leading=9)
    toc = TableOfContents()
    toc.levelStyles = [ParagraphStyle('ContentsEntry', parent=body, fontSize=9, leading=13)]
    story = [Paragraph('Hephaestus', ParagraphStyle('TitleCustom', parent=heading,
               fontSize=30, leading=35)), Paragraph('Autonomous Invention Harness · version 1.2', subheading),
             Paragraph('30 September 2026 · Implementation baseline. No invention runtime or scientific campaign has been validated by this package.', body),
             Paragraph('Reader edition: master specification and four normative supplements. The full obligation matrix, acceptance cases, schemas and recorded checks accompany this PDF in the package.', body),
             Spacer(1, 10), Paragraph('Contents', heading), toc, PageBreak()]
    parser = MarkdownIt('commonmark').enable('table')
    bookmarks = 0
    for source_number, source in enumerate(SOURCES):
        if source_number:
            story.append(PageBreak())
            title = Paragraph(f'Appendix {source_number}: ' +
                              {'docs/QUALIFICATION_CONTRACT.md': 'Qualification and lineage',
                               'docs/CONTEXT_ASSEMBLY_CAMPAIGN.md': 'First domain and campaign',
                               'docs/RETRIEVAL_AND_MEMORY.md': 'Retrieval and memory security',
                               'docs/SELF_IMPROVEMENT.md': 'Required autonomous self-improvement'}[source], heading)
            title.bookmark_key = f'appendix-{source_number}'
            story.append(title)
            bookmarks += 1
        tokens = parser.parse((ROOT / source).read_text())
        i = 0
        while i < len(tokens):
            token = tokens[i]
            if token.type == 'heading_open':
                content = tokens[i + 1]
                if source_number and token.tag == 'h1':
                    i += 3
                    continue
                paragraph = Paragraph(inline(content.children), heading if token.tag == 'h1' else subheading)
                if source_number == 0 and re.match(r'^\d+\.', content.content):
                    paragraph.bookmark_key = 'section-' + content.content.split('.')[0]
                    bookmarks += 1
                story.append(paragraph)
                i += 3
                continue
            if token.type == 'inline':
                story.append(Paragraph(inline(token.children), body))
            elif token.type in {'fence', 'code_block'}:
                story.append(Preformatted(token.content.rstrip(), code, maxLineLength=96))
                story.append(Spacer(1, 8))
            elif token.type == 'table_open':
                rows, row = [], []
                i += 1
                while tokens[i].type != 'table_close':
                    if tokens[i].type == 'tr_open': row = []
                    elif tokens[i].type == 'inline': row.append(Paragraph(inline(tokens[i].children), cell))
                    elif tokens[i].type == 'tr_close': rows.append(row)
                    i += 1
                table = LongTable(rows, colWidths=[(WIDTH - 2 * MARGIN) / len(rows[0])] * len(rows[0]), repeatRows=1)
                table.setStyle(TableStyle([('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#EAF0F5')),
                    ('VALIGN', (0, 0), (-1, -1), 'TOP'), ('GRID', (0, 0), (-1, -1), .3, colors.HexColor('#CCD5DD')),
                    ('LEFTPADDING', (0, 0), (-1, -1), 6), ('RIGHTPADDING', (0, 0), (-1, -1), 6),
                    ('TOPPADDING', (0, 0), (-1, -1), 6), ('BOTTOMPADDING', (0, 0), (-1, -1), 6)]))
                story += [table, Spacer(1, 10)]
            i += 1
    path = ROOT / 'MASTER_SPEC.pdf'
    ReaderDoc(path).multiBuild(story)
    document = pymupdf.open(path)
    flags, links = [], 0
    render_dir = Path(tempfile.mkdtemp(prefix='hephaestus-pdf-v1.2-'))
    for page in document:
        page.get_pixmap(matrix=pymupdf.Matrix(.8, .8)).save(render_dir / f'page-{page.number + 1:03}.png')
        links += sum(1 for link in page.get_links() if link.get('uri'))
        for block in page.get_text('dict')['blocks']:
            for line in block.get('lines', []):
                for span in line['spans']:
                    rect = pymupdf.Rect(span['bbox'])
                    if not page.rect.contains(rect):
                        flags.append({'page': page.number + 1, 'text': span['text'], 'bbox': list(rect)})
    def normalized(value):
        return ''.join(c for c in value if not c.isspace()).replace('\u00ad', '')
    extracted = normalized(''.join(page.get_text(clip=pymupdf.Rect(
        0, 0, page.rect.width, page.rect.height - MARGIN)) for page in document))
    missing, checked = [], 0
    for source in SOURCES:
        tokens = parser.parse((ROOT / source).read_text())
        for i, token in enumerate(tokens):
            if token.type != 'inline': continue
            # Appendix title replaces the duplicate document H1.
            if (source != 'MASTER_SPEC.md' and i and tokens[i - 1].type == 'heading_open'
                    and tokens[i - 1].tag == 'h1'): continue
            plain = ''.join(t.content for t in token.children if t.type in {'text', 'code_inline'})
            if len(plain) > 30:
                checked += 1
                if normalized(plain) not in extracted:
                    missing.append({'source': source, 'text': plain[:150]})
    result = {'pages': len(document), 'bookmarked_sections': len(document.get_toc()),
              'expected_bookmarks': bookmarks, 'source_links': links,
              'text_outside_page_bounds_flags': len(flags), 'flags': flags,
              'render_directory': str(render_dir), 'rendering': 'Every page rendered; visual inspection is reported separately.',
              'source_documents': SOURCES,
              'checked_text_blocks': checked, 'missing_text_blocks': missing,
              'limitations': 'Document rendering/integrity only; no runtime or scientific validation.'}
    (ROOT / 'validation/pdf-layout.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
    if flags or missing or len(document.get_toc()) != bookmarks:
        raise SystemExit('PDF layout/bookmark verification failed')

if __name__ == '__main__':
    build()
