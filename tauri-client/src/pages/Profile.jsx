import { useState } from 'react'
import { api } from '../api'
import { useAuth } from '../AuthContext'

export default function ProfilePage({ onBack }) {
  const { user, login, logout } = useAuth()
  const [nickname, setNickname] = useState(user?.nickname || '')
  const [password, setPassword] = useState('')
  const [saving, setSaving] = useState(false)
  const [msg, setMsg] = useState(null)

  async function save(e) {
    e.preventDefault()
    setSaving(true)
    setMsg(null)
    try {
      const body = {}
      if (nickname.trim() && nickname !== user?.nickname) body.nickname = nickname.trim()
      if (password) body.password = password
      if (!Object.keys(body).length) { setMsg({ text: '没有修改', err: false }); setSaving(false); return }
      await api.updateProfile(body)
      const updated = await api.getProfile()
      login(updated, localStorage.getItem('token'))
      setPassword('')
      setMsg({ text: '保存成功', err: false })
    } catch (e) {
      setMsg({ text: e.message, err: true })
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="flex-col gap-16" style={{ maxWidth: 480, margin: '0 auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <button className="btn btn-ghost btn-sm" onClick={onBack}>← 返回</button>
        <span style={{ fontSize: 17, fontWeight: 600 }}>个人中心</span>
      </div>

      <div className="card" style={{ padding: 24 }}>
        <div style={{ position: 'relative', zIndex: 1 }}>
          <div style={{ marginBottom: 20 }}>
            <div style={{ fontSize: 13, color: 'var(--text-secondary)' }}>用户名</div>
            <div style={{ fontSize: 16, fontWeight: 600, marginTop: 4 }}>@{user?.username}</div>
          </div>
          <div style={{ marginBottom: 20 }}>
            <div style={{ fontSize: 13, color: 'var(--text-secondary)' }}>角色</div>
            <span className={`role-badge ${user?.role === 'admin' ? 'admin' : ''}`} style={{ marginTop: 4, display: 'inline-block' }}>
              {user?.role}
            </span>
          </div>

          <div className="divider" />

          <form onSubmit={save} className="flex-col gap-12" style={{ marginTop: 16 }}>
            <div className="form-group">
              <label className="label">昵称</label>
              <input className="input" value={nickname} onChange={e => setNickname(e.target.value)} />
            </div>
            <div className="form-group">
              <label className="label">新密码（留空不修改）</label>
              <input className="input" type="password" value={password} onChange={e => setPassword(e.target.value)} placeholder="至少 6 位" />
            </div>

            {msg && (
              <div style={{ fontSize: 13, color: msg.err ? 'var(--danger)' : 'var(--success)' }}>{msg.text}</div>
            )}

            <div style={{ display: 'flex', gap: 10, marginTop: 4 }}>
              <button className="btn btn-primary" type="submit" disabled={saving}>
                {saving ? <span className="spinner" style={{ width: 14, height: 14 }} /> : '保存'}
              </button>
              <button className="btn btn-ghost" type="button" onClick={logout}>退出登录</button>
            </div>
          </form>
        </div>
      </div>
    </div>
  )
}
