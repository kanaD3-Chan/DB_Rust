import { useState, useEffect } from 'react'
import { api } from '../api'
import { useAuth } from '../AuthContext'

export default function MembersPage({ onBack }) {
  const { user } = useAuth()
  const [members, setMembers] = useState([])
  const [loading, setLoading] = useState(true)
  const [err, setErr] = useState('')

  async function load() {
    setLoading(true)
    try {
      const data = await api.listMembers()
      setMembers(data)
    } catch (e) {
      setErr(e.message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  async function toggleRole(m) {
    const newRole = m.role === 'admin' ? 'member' : 'admin'
    if (!confirm(`将 ${m.nickname} 的角色改为 ${newRole}？`)) return
    try {
      await api.updateMemberRole(m.id, newRole)
      load()
    } catch (e) {
      setErr(e.message)
    }
  }

  async function deleteMember(m) {
    if (!confirm(`确定删除成员 ${m.nickname}？`)) return
    try {
      await api.deleteMember(m.id)
      load()
    } catch (e) {
      setErr(e.message)
    }
  }

  return (
    <div className="flex-col gap-16" style={{ maxWidth: 700, margin: '0 auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <button className="btn btn-ghost btn-sm" onClick={onBack}>← 返回</button>
        <span style={{ fontSize: 17, fontWeight: 600 }}>成员管理</span>
      </div>

      {err && <div style={{ color: 'var(--danger)', fontSize: 13 }}>{err}</div>}

      <div className="card">
        {loading ? (
          <div className="empty-state"><span className="spinner" /></div>
        ) : members.length === 0 ? (
          <div className="empty-state">暂无成员</div>
        ) : (
          <div style={{ position: 'relative', zIndex: 1 }}>
            {members.map(m => (
              <div key={m.id} style={{
                display: 'flex', alignItems: 'center', gap: 12,
                padding: '12px 16px', borderBottom: '1px solid var(--border)',
              }}>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontWeight: 600, fontSize: 14 }}>{m.nickname}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-secondary)' }}>@{m.username}</div>
                </div>
                <span className={`role-badge ${m.role === 'admin' ? 'admin' : ''}`}>{m.role}</span>
                <span style={{ fontSize: 12, color: 'var(--text-tertiary)' }}>{m.created_at?.slice(0, 10)}</span>
                {user?.role === 'admin' && m.id !== user.id && (
                  <div style={{ display: 'flex', gap: 6 }}>
                    <button className="btn btn-ghost btn-sm" onClick={() => toggleRole(m)}>
                      {m.role === 'admin' ? '降为成员' : '升为管理员'}
                    </button>
                    <button className="btn btn-danger btn-sm" onClick={() => deleteMember(m)}>删除</button>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
