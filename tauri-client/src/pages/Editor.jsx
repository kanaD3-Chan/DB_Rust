import { useState } from 'react'
import { api } from '../api'
import Markdown from '../components/Markdown'

const CATS = ['general', 'tech', 'security', 'life']

export default function EditorPage({ article, onSaved, onCancel }) {
  const [title, setTitle] = useState(article?.title || '')
  const [content, setContent] = useState(article?.content || '')
  const [category, setCategory] = useState(article?.category || 'general')
  const [preview, setPreview] = useState(false)
  const [saving, setSaving] = useState(false)
  const [err, setErr] = useState('')

  async function save() {
    if (!title.trim() || !content.trim()) { setErr('标题和内容不能为空'); return }
    setSaving(true)
    setErr('')
    try {
      if (article?.id) {
        await api.updateArticle(article.id, { title, content, category })
      } else {
        await api.createArticle(title, content, category)
      }
      onSaved()
    } catch (e) {
      setErr(e.message)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="flex-col gap-12" style={{ height: '100%' }}>
      {/* toolbar */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexShrink: 0 }}>
        <button className="btn btn-ghost btn-sm" onClick={onCancel}>取消</button>
        <div style={{ flex: 1 }} />
        <button
          className={`btn btn-sm ${preview ? 'btn-primary' : 'btn-ghost'}`}
          onClick={() => setPreview(p => !p)}
        >
          {preview ? '编辑' : '预览'}
        </button>
        <select className="select" style={{ height: 32, fontSize: 13 }} value={category} onChange={e => setCategory(e.target.value)}>
          {CATS.map(c => <option key={c} value={c}>{c}</option>)}
        </select>
        <button className="btn btn-primary btn-sm" onClick={save} disabled={saving}>
          {saving ? <span className="spinner" style={{ width: 14, height: 14 }} /> : '发布'}
        </button>
      </div>

      {/* title */}
      <input
        className="input"
        style={{ fontSize: 18, fontWeight: 600, padding: '10px 16px' }}
        placeholder="文章标题…"
        value={title}
        onChange={e => setTitle(e.target.value)}
      />

      {err && <div style={{ color: 'var(--danger)', fontSize: 13 }}>{err}</div>}

      {/* content area */}
      <div className="card" style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
        <div style={{ position: 'relative', zIndex: 1, flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          {preview ? (
            <div style={{ padding: 20, overflowY: 'auto', flex: 1 }}>
              {content
                ? <Markdown>{content}</Markdown>
                : <span style={{ color: 'var(--text-tertiary)' }}>暂无内容</span>
              }
            </div>
          ) : (
            <textarea
              className="textarea"
              style={{ flex: 1, resize: 'none', borderRadius: 0, border: 'none', padding: 20, minHeight: 0, fontFamily: "'Share Tech Mono', 'Courier New', monospace", fontSize: 14, lineHeight: 1.6 }}
              placeholder="用 Markdown 写点什么…"
              value={content}
              onChange={e => setContent(e.target.value)}
            />
          )}
        </div>
      </div>
    </div>
  )
}
