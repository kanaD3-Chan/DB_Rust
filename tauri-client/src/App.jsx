import { useState } from 'react'
import { ThemeProvider, useTheme } from './ThemeContext'
import { AuthProvider, useAuth } from './AuthContext'
import LoginPage from './pages/Login'
import HomePage from './pages/Home'
import ArticlePage from './pages/Article'
import EditorPage from './pages/Editor'
import MembersPage from './pages/Members'
import ProfilePage from './pages/Profile'
import './index.css'

function ThemeSwitcher() {
  const { theme, setTheme, themes } = useTheme()
  return (
    <div className="theme-switcher">
      {themes.map(t => (
        <div
          key={t.id}
          className={`theme-dot ${theme === t.id ? 'active' : ''}`}
          title={t.label}
          style={{
            background: t.dot || t.color,
            borderColor: theme === t.id ? t.border : 'transparent',
            boxShadow: theme === t.id ? `0 0 0 1px ${t.border}40` : 'none',
          }}
          onClick={() => setTheme(t.id)}
        />
      ))}
    </div>
  )
}

function Shell() {
  const { user, logout } = useAuth()
  const { theme } = useTheme()
  const [page, setPage] = useState('home')
  const [articleId, setArticleId] = useState(null)
  const [editArticle, setEditArticle] = useState(null)
  const [status, setStatus] = useState(null)

  const isCyber = theme === 'cyberpunk'

  function showStatus(text, isErr = false) {
    setStatus({ text, isErr })
    setTimeout(() => setStatus(null), 3000)
  }

  function openArticle(id) { setArticleId(id); setPage('article') }
  function openEditor(article = null) { setEditArticle(article); setPage('editor') }
  function goHome() { setPage('home'); setArticleId(null); setEditArticle(null) }

  if (!user) return <LoginPage />

  const navItems = [
    { id: 'home', label: isCyber ? '[ 首页 ]' : '首页', icon: '◈' },
    ...(user.role === 'admin' ? [{ id: 'members', label: isCyber ? '[ 成员 ]' : '成员管理', icon: '◉' }] : []),
    { id: 'profile', label: isCyber ? '[ 我的 ]' : '个人中心', icon: '◎' },
  ]

  return (
    <div className="app-shell">
      <div className="topbar">
        <span className="topbar-title">
          {isCyber ? '▸ SLSEC FORUM ◂' : 'SLsec 论坛'}
        </span>
        <ThemeSwitcher />
        <div className="user-badge">
          <span>{user.nickname}</span>
          <span className={`role-badge ${user.role === 'admin' ? 'admin' : ''}`}>{user.role}</span>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={logout}>退出</button>
      </div>

      <div className="main-area">
        <div className="sidebar">
          <div className="sidebar-section">导航</div>
          {navItems.map(item => (
            <button
              key={item.id}
              className={`sidebar-item ${page === item.id ? 'active' : ''}`}
              onClick={() => { setPage(item.id); setArticleId(null); setEditArticle(null) }}
            >
              <span style={{ fontSize: 12 }}>{item.icon}</span>
              {item.label}
            </button>
          ))}
        </div>

        <div className="content">
          {page === 'home' && (
            <HomePage
              onOpenArticle={openArticle}
              onNewArticle={() => openEditor(null)}
            />
          )}
          {page === 'article' && articleId && (
            <ArticlePage
              articleId={articleId}
              onBack={goHome}
              onEdit={(a) => openEditor(a)}
            />
          )}
          {page === 'editor' && (
            <EditorPage
              article={editArticle}
              onSaved={() => { showStatus('保存成功'); goHome() }}
              onCancel={() => articleId ? setPage('article') : goHome()}
            />
          )}
          {page === 'members' && (
            <MembersPage onBack={goHome} />
          )}
          {page === 'profile' && (
            <ProfilePage onBack={goHome} />
          )}
        </div>
      </div>

      <div className={`status-bar ${status?.isErr ? 'error' : status ? 'success' : ''}`}>
        {status?.text || (isCyber ? '▸ SYSTEM READY ◂' : '就绪')}
      </div>
    </div>
  )
}

export default function App() {
  return (
    <ThemeProvider>
      <AuthProvider>
        <Shell />
      </AuthProvider>
    </ThemeProvider>
  )
}
