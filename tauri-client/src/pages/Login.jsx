import { useState } from 'react'
import { api } from '../api'
import { useAuth } from '../AuthContext'
import { useTheme } from '../ThemeContext'

export default function LoginPage() {
  const { login } = useAuth()
  const { theme } = useTheme()
  const [mode, setMode] = useState('login')
  const [form, setForm] = useState({ username: '', nickname: '', password: '' })
  const [err, setErr] = useState('')
  const [loading, setLoading] = useState(false)

  const set = (k) => (e) => setForm(f => ({ ...f, [k]: e.target.value }))

  async function submit(e) {
    e.preventDefault()
    setErr('')
    setLoading(true)
    try {
      let data
      if (mode === 'login') {
        data = await api.login(form.username, form.password)
      } else {
        data = await api.register(form.username, form.nickname, form.password)
      }
      login(data.user, data.token)
    } catch (e) {
      setErr(e.message)
    } finally {
      setLoading(false)
    }
  }

  const isCyber = theme === 'cyberpunk'
  const isAero = theme === 'frutiger-aero'

  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      height: '100%',
      padding: 20,
    }}>
      <div className="card" style={{
        width: 380,
        padding: 36,
        ...(isAero ? { background: 'rgba(255,255,255,0.7)' } : {}),
      }}>
        <div style={{ position: 'relative', zIndex: 1 }}>
          {isCyber && (
            <div style={{ textAlign: 'center', marginBottom: 24, color: 'var(--accent)', fontSize: 11, letterSpacing: 4, textTransform: 'uppercase', textShadow: 'var(--glow, 0 0 8px rgba(0,255,255,0.6))' }}>
              ▸ SLsec Forum System ◂
            </div>
          )}

          <h2 style={{
            fontSize: isCyber ? 13 : 22,
            fontWeight: 700,
            marginBottom: 24,
            textAlign: 'center',
            color: isCyber ? 'var(--accent)' : 'var(--text)',
            textTransform: isCyber ? 'uppercase' : 'none',
            letterSpacing: isCyber ? 3 : -0.5,
            textShadow: isCyber ? '0 0 8px rgba(0,255,255,0.6)' : 'none',
          }}>
            {isCyber ? '[ ' : ''}{mode === 'login' ? '登录' : '注册'}{isCyber ? ' ]' : ''}
          </h2>

          <div style={{ display: 'flex', gap: 8, marginBottom: 24 }}>
            {['login', 'register'].map(m => (
              <button
                key={m}
                className={`btn w-full ${mode === m ? 'btn-primary' : 'btn-ghost'}`}
                onClick={() => { setMode(m); setErr('') }}
              >
                {m === 'login' ? '登录' : '注册'}
              </button>
            ))}
          </div>

          <form onSubmit={submit} className="flex-col gap-12">
            <div className="form-group">
              <label className="label">用户名</label>
              <input className="input" value={form.username} onChange={set('username')} placeholder="username" autoComplete="username" />
            </div>

            {mode === 'register' && (
              <div className="form-group">
                <label className="label">昵称</label>
                <input className="input" value={form.nickname} onChange={set('nickname')} placeholder="nickname" />
              </div>
            )}

            <div className="form-group">
              <label className="label">密码</label>
              <input className="input" type="password" value={form.password} onChange={set('password')} placeholder="••••••" autoComplete="current-password" />
            </div>

            {err && <div style={{ color: 'var(--danger)', fontSize: 13 }}>{err}</div>}

            <button className="btn btn-primary w-full" type="submit" disabled={loading} style={{ marginTop: 4 }}>
              {loading ? <span className="spinner" style={{ width: 16, height: 16 }} /> : (mode === 'login' ? '登录' : '注册')}
            </button>
          </form>
        </div>
      </div>
    </div>
  )
}
