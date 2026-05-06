const escapeHtml = (value: string) =>
  value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');

const renderInlineMarkdown = (value: string) => {
  let html = escapeHtml(value);
  html = html.replace(/~~(.+?)~~/g, '<del>$1</del>');
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/&lt;u&gt;(.+?)&lt;\/u&gt;/g, '<u>$1</u>');
  return html;
};

export const renderMarkdown = (markdown: string) => {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n');
  const blocks: string[] = [];
  let listItems: string[] = [];

  const flushList = () => {
    if (!listItems.length) return;
    blocks.push(`<ul>${listItems.map((item) => `<li>${item}</li>`).join('')}</ul>`);
    listItems = [];
  };

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();
    if (!line.trim()) {
      flushList();
      continue;
    }

    const listMatch = line.match(/^\s*[-*]\s+(.+)$/);
    if (listMatch) {
      listItems.push(renderInlineMarkdown(listMatch[1]));
      continue;
    }

    flushList();

    if (line.startsWith('### ')) {
      blocks.push(`<h3>${renderInlineMarkdown(line.slice(4).trim())}</h3>`);
    } else if (line.startsWith('## ')) {
      blocks.push(`<h2>${renderInlineMarkdown(line.slice(3).trim())}</h2>`);
    } else if (line.startsWith('# ')) {
      blocks.push(`<h1>${renderInlineMarkdown(line.slice(2).trim())}</h1>`);
    } else {
      blocks.push(`<p>${renderInlineMarkdown(line.trim())}</p>`);
    }
  }

  flushList();
  return blocks.join('');
};
