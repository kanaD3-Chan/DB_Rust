import { createContext, useContext, useState, useEffect } from 'react'

const THEMES = [
  { id: 'apple-light',   label: 'Apple Light',    color: '#f2f2f7', dot: 'linear-gradient(135deg,#fff 0%,#e5e5ea 100%)', border: '#007aff' },
  { id: 'apple-dark',    label: 'Apple Dark',     color: '#1c1c1e', dot: 'linear-gradient(135deg,#2c2c2e 0%,#000 100%)', border: '#0a84ff' },
  { id: 'frutiger-aero', label: 'Frutiger Aero',  color: '#b8dff0', dot: 'linear-gradient(135deg,#a0d8ef 0%,#b8e8c8 60%,#d0eeff 100%)', border: '#0077bb' },
  { id: 'cyberpunk',     label: 'Cyberpunk',      color: '#060610', dot: 'linear-gradient(135deg,#060610 0%,#001a2a 60%,#00f0ff44 100%)', border: '#00f0ff' },
]

const ThemeCtx = createContext(null)

export function ThemeProvider({ children }) {
  const [theme, setTheme] = useState(() => localStorage.getItem('theme') || 'apple-light')

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
    localStorage.setItem('theme', theme)
  }, [theme])

  return (
    <ThemeCtx.Provider value={{ theme, setTheme, themes: THEMES }}>
      {children}
    </ThemeCtx.Provider>
  )
}

export const useTheme = () => useContext(ThemeCtx)
