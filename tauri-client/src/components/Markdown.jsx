import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import rehypeHighlight from 'rehype-highlight'
import rehypeSanitize, { defaultSchema } from 'rehype-sanitize'
import '../hljs-themes.css'

// 在默认白名单基础上允许 highlight.js 注入的 class，其余全部剥离
const sanitizeSchema = {
  ...defaultSchema,
  attributes: {
    ...defaultSchema.attributes,
    code: [...(defaultSchema.attributes?.code ?? []), 'className'],
    span: [...(defaultSchema.attributes?.span ?? []), 'className'],
  },
}

function safeHref(href) {
  if (!href) return '#'
  const lower = href.trim().toLowerCase()
  if (lower.startsWith('javascript:') || lower.startsWith('vbscript:') || lower.startsWith('data:')) return '#'
  return href
}

export default function Markdown({ children }) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      // sanitize 必须在 highlight 之后，否则 class 会被剥掉
      rehypePlugins={[rehypeHighlight, [rehypeSanitize, sanitizeSchema]]}
      components={{
        h1: ({ children }) => <h1 style={{ fontSize: 22, fontWeight: 700, margin: '20px 0 10px', color: 'var(--text)' }}>{children}</h1>,
        h2: ({ children }) => <h2 style={{ fontSize: 18, fontWeight: 600, margin: '18px 0 8px', color: 'var(--text)' }}>{children}</h2>,
        h3: ({ children }) => <h3 style={{ fontSize: 16, fontWeight: 600, margin: '14px 0 6px', color: 'var(--text)' }}>{children}</h3>,
        p: ({ children }) => <p style={{ margin: '8px 0', lineHeight: 1.7, color: 'var(--text)' }}>{children}</p>,
        a: ({ href, children }) => (
          <a href={safeHref(href)} target="_blank" rel="noreferrer noopener" style={{ color: 'var(--accent)', textDecoration: 'none' }}>
            {children}
          </a>
        ),
        code: ({ inline, className, children }) =>
          inline
            ? <code style={{ background: 'var(--bg-input)', padding: '2px 6px', borderRadius: 4, fontSize: 13, fontFamily: "'Share Tech Mono', monospace", color: 'var(--text)' }}>{children}</code>
            : <code className={className}>{children}</code>,
        pre: ({ children }) => (
          <pre style={{ background: 'var(--bg-code)', borderRadius: 'var(--radius-sm)', padding: '14px 16px', overflowX: 'auto', margin: '10px 0', fontSize: 13 }}>
            {children}
          </pre>
        ),
        blockquote: ({ children }) => (
          <blockquote style={{ borderLeft: '3px solid var(--accent)', paddingLeft: 14, margin: '10px 0', color: 'var(--text-secondary)', fontStyle: 'italic' }}>
            {children}
          </blockquote>
        ),
        ul: ({ children }) => <ul style={{ paddingLeft: 20, margin: '8px 0', lineHeight: 1.7 }}>{children}</ul>,
        ol: ({ children }) => <ol style={{ paddingLeft: 20, margin: '8px 0', lineHeight: 1.7 }}>{children}</ol>,
        li: ({ children }) => <li style={{ margin: '3px 0', color: 'var(--text)' }}>{children}</li>,
        table: ({ children }) => (
          <div style={{ overflowX: 'auto', margin: '10px 0' }}>
            <table style={{ borderCollapse: 'collapse', width: '100%', fontSize: 14 }}>{children}</table>
          </div>
        ),
        th: ({ children }) => <th style={{ padding: '8px 12px', borderBottom: '2px solid var(--border)', textAlign: 'left', color: 'var(--text-secondary)', fontWeight: 600 }}>{children}</th>,
        td: ({ children }) => <td style={{ padding: '8px 12px', borderBottom: '1px solid var(--border)', color: 'var(--text)' }}>{children}</td>,
        hr: () => <hr style={{ border: 'none', borderTop: '1px solid var(--border)', margin: '16px 0' }} />,
        img: ({ src, alt }) => {
          const safe = safeHref(src)
          return safe === '#'
            ? null
            : <img src={safe} alt={alt ?? ''} style={{ maxWidth: '100%', borderRadius: 'var(--radius-sm)', margin: '8px 0' }} />
        },
      }}
    >
      {children}
    </ReactMarkdown>
  )
}
