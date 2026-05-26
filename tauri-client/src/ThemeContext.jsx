import { createContext, useContext, useState, useEffect } from 'react'

const THEMES = [
  { id: 'apple-light', label: 'Apple Light', color: '#f5f5f7', border: '#007aff' },
  { id: 'apple-dark',  label: 'Apple Dark',  color: '#1c1c1e', border: '#0a84ff' },
  { id: 'cyberpunk',   label: 'Cyberpunk',   color: '#0a0a0f', border: '#00ffff' },
  { id: 'frutiger-aero', label: 'Frutiger Aero', color: '#a8d8ea', border: '#0088cc' },
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
