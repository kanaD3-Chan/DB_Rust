import { useState, useEffect, useCallback } from 'react'
import { api } from '../api'
import { useAuth } from '../AuthContext'

const CATS = [
  { value: '', label: '全部' },
  { value: 'general', label: '综合' },
  { value: 'tech', label: '技术' },
  { value: 'security', label: '安全' },
  { value: 'life', label: '生活' },
]

export default function HomePage({ onOpenArticle, onNewArticle }) {
  const { user } = useAuth()
  const [articles, setArticles] = useState([])
  const [loading, setLoading] = useState(false)
  const [cat, setCat] = useState('')
  const [search, setSearch] = useState('')
  const [searchInput, setSearchInput] = useState('')

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const data = await api.listArticles({ category: cat || undefined, search: search || undefined, page_size: 200 })
      setArticles(data.articles)
    } catch {
      setArticles([])
    } finally {
      setLoading(false)
    }
  }, [cat, search])

  useEffect(() => { load() }, [load])

  function handleSearch(e) {
    e.preventDefault()
    setSearch(searchInput)
  }

  const catColor = { security: 'var(--danger)', tech: '#007aff', life: 'var(--success)', general: 'var(--text-tertiary)' }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', gap: 0 }}>
      {/* toolbar */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 0', flexShrink: 0, flexWrap: 'wrap' }}>
        <div className="cat-tabs">
          {CATS.map(c => (
            <button key={c.value} className={`cat-tab ${cat === c.value ? 'active' : ''}`} onClick={() => setCat(c.value)}>
              {c.label}
            </button>
          ))}
        </div>

        <form onSubmit={handleSearch} className="search-bar" style={{ marginLeft: 'auto' }}>
          <input
            className="input"
            style={{ height: 32, padding: '4px 12px', fontSize: 13 }}
            placeholder="搜索文章…"
            value={searchInput}
            onChange={e => setSearchInput(e.target.value)}
          />
          <button className="btn btn-ghost btn-sm" type="submit">搜索</button>
        </form>

        {user && (
          <button className="btn btn-primary btn-sm" onClick={onNewArticle}>
            + 写文章
          </button>
        )}
      </div>

      {/* list */}
      <div className="card" style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
        {loading ? (
          <div className="empty-state"><span className="spinner" /></div>
        ) : articles.length === 0 ? (
          <div className="empty-state">暂无文章</div>
        ) : (
          <div style={{ overflowY: 'auto', flex: 1 }}>
            {articles.map(a => (
              <div key={a.id} className="article-item" onClick={() => onOpenArticle(a.id)}>
                <div style={{ display: 'flex', alignItems: 'flex-start', gap: 8 }}>
                  {a.pinned && <span style={{ fontSize: 12, color: 'var(--danger)', flexShrink: 0, marginTop: 2 }}>📌</span>}
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div className="article-title" style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {a.title}
                    </div>
                    <div className="article-meta">
                      <span
                        className="tag"
                        style={{ background: `${catColor[a.category] || 'var(--tag-bg)'}18`, color: catColor[a.category] || 'var(--tag-text)' }}
                      >
                        {a.category}
                      </span>
                      <span>{a.author_nickname}</span>
                      <span>👁 {a.view_count}</span>
                      <span>💬 {a.comment_count}</span>
                      <span style={{ marginLeft: 'auto' }}>{a.created_at?.slice(0, 10)}</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
