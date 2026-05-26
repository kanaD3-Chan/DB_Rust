import { useState, useEffect } from 'react'
import { api } from '../api'
import { useAuth } from '../AuthContext'
import Markdown from '../components/Markdown'

export default function ArticlePage({ articleId, onBack, onEdit }) {
  const { user } = useAuth()
  const [article, setArticle] = useState(null)
  const [comments, setComments] = useState([])
  const [loading, setLoading] = useState(true)
  const [commentInput, setCommentInput] = useState('')
  const [replyTo, setReplyTo] = useState(null)
  const [submitting, setSubmitting] = useState(false)
  const [err, setErr] = useState('')

  useEffect(() => {
    setLoading(true)
    Promise.all([api.getArticle(articleId), api.listComments(articleId)])
      .then(([a, c]) => { setArticle(a); setComments(c) })
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [articleId])

  async function submitComment(e) {
    e.preventDefault()
    if (!commentInput.trim()) return
    setSubmitting(true)
    setErr('')
    try {
      await api.createComment(articleId, commentInput.trim(), replyTo)
      setCommentInput('')
      setReplyTo(null)
      const c = await api.listComments(articleId)
      setComments(c)
    } catch (e) {
      setErr(e.message)
    } finally {
      setSubmitting(false)
    }
  }

  async function deleteArticle() {
    if (!confirm('确定删除这篇文章？')) return
    try {
      await api.deleteArticle(articleId)
      onBack()
    } catch (e) {
      setErr(e.message)
    }
  }

  const canEdit = user && article && (user.id === article.author_id || user.role === 'admin')

  const topComments = comments.filter(c => !c.parent_id)
  const replies = (pid) => comments.filter(c => c.parent_id === pid)

  if (loading) return <div className="empty-state"><span className="spinner" /></div>
  if (!article) return <div className="empty-state">文章不存在</div>

  return (
    <div className="flex-col gap-16" style={{ maxWidth: 800, margin: '0 auto' }}>
      {/* header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <button className="btn btn-ghost btn-sm" onClick={onBack}>← 返回</button>
        <div style={{ flex: 1 }} />
        {canEdit && (
          <>
            <button className="btn btn-ghost btn-sm" onClick={() => onEdit(article)}>编辑</button>
            <button className="btn btn-danger btn-sm" onClick={deleteArticle}>删除</button>
          </>
        )}
      </div>

      {/* article */}
      <div className="card" style={{ padding: 28 }}>
        <div style={{ position: 'relative', zIndex: 1 }}>
          <h1 style={{ fontSize: 22, fontWeight: 700, marginBottom: 12, lineHeight: 1.3, color: 'var(--text)' }}>
            {article.title}
          </h1>
          <div className="article-meta" style={{ marginBottom: 20 }}>
            <span className="tag">{article.category}</span>
            <span>{article.author_nickname}</span>
            <span>👁 {article.view_count}</span>
            <span>💬 {article.comment_count}</span>
            <span>{article.created_at?.slice(0, 10)}</span>
          </div>
          <div className="divider" />
          <div style={{ marginTop: 16 }}>
            <Markdown>{article.content}</Markdown>
          </div>
        </div>
      </div>

      {/* comments */}
      <div className="card" style={{ padding: 20 }}>
        <div style={{ position: 'relative', zIndex: 1 }}>
          <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>
            评论 ({comments.length})
          </div>

          {topComments.length === 0 ? (
            <div className="empty-state" style={{ padding: '24px 0' }}>暂无评论</div>
          ) : (
            topComments.map(c => (
              <div key={c.id} className="comment-item">
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span className="comment-author">{c.author_nickname}</span>
                  <span className="comment-meta">{c.created_at?.slice(0, 16)}</span>
                  {user && (
                    <button
                      className="btn btn-ghost btn-sm"
                      style={{ marginLeft: 'auto', fontSize: 12, padding: '2px 8px' }}
                      onClick={() => setReplyTo(c.id)}
                    >
                      回复
                    </button>
                  )}
                </div>
                <div className="comment-content">{c.content}</div>

                {replies(c.id).map(r => (
                  <div key={r.id} className="reply-indent" style={{ marginTop: 8 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                      <span className="comment-author">{r.author_nickname}</span>
                      <span className="comment-meta">{r.created_at?.slice(0, 16)}</span>
                    </div>
                    <div className="comment-content">{r.content}</div>
                  </div>
                ))}
              </div>
            ))
          )}

          {user ? (
            <form onSubmit={submitComment} className="flex-col gap-8" style={{ marginTop: 16 }}>
              {replyTo && (
                <div style={{ fontSize: 12, color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', gap: 8 }}>
                  回复评论 #{replyTo}
                  <button type="button" className="btn btn-ghost btn-sm" style={{ padding: '1px 8px', fontSize: 11 }} onClick={() => setReplyTo(null)}>取消</button>
                </div>
              )}
              <textarea
                className="textarea"
                style={{ minHeight: 80 }}
                placeholder="写下你的评论…"
                value={commentInput}
                onChange={e => setCommentInput(e.target.value)}
              />
              {err && <div style={{ color: 'var(--danger)', fontSize: 13 }}>{err}</div>}
              <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                <button className="btn btn-primary btn-sm" type="submit" disabled={submitting}>
                  {submitting ? <span className="spinner" style={{ width: 14, height: 14 }} /> : '发表评论'}
                </button>
              </div>
            </form>
          ) : (
            <div style={{ marginTop: 16, fontSize: 13, color: 'var(--text-secondary)' }}>登录后可发表评论</div>
          )}
        </div>
      </div>
    </div>
  )
}
