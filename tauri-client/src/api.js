const BASE = 'http://172.16.173.140:3000'

let _token = localStorage.getItem('token') || ''

export function setToken(t) {
  _token = t
  localStorage.setItem('token', t)
}

export function clearToken() {
  _token = ''
  localStorage.removeItem('token')
  localStorage.removeItem('user')
}

export function getToken() { return _token }

async function req(method, path, body) {
  const headers = { 'Content-Type': 'application/json' }
  if (_token) headers['Authorization'] = `Bearer ${_token}`
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  })
  const json = await res.json()
  if (!json.success) throw new Error(json.error || '请求失败')
  return json.data
}

export const api = {
  register: (username, nickname, password) =>
    req('POST', '/api/register', { username, nickname, password }),

  login: (username, password) =>
    req('POST', '/api/login', { username, password }),

  listArticles: (params = {}) => {
    const q = new URLSearchParams()
    if (params.page) q.set('page', params.page)
    if (params.page_size) q.set('page_size', params.page_size)
    if (params.category) q.set('category', params.category)
    if (params.search) q.set('search', params.search)
    return req('GET', `/api/articles?${q}`)
  },

  getArticle: (id) => req('GET', `/api/articles/${id}`),

  createArticle: (title, content, category) =>
    req('POST', '/api/articles', { title, content, category }),

  updateArticle: (id, data) => req('PUT', `/api/articles/${id}`, data),

  deleteArticle: (id) => req('DELETE', `/api/articles/${id}`),

  listComments: (articleId) => req('GET', `/api/articles/${articleId}/comments`),

  createComment: (articleId, content, parent_id) =>
    req('POST', `/api/articles/${articleId}/comments`, { content, parent_id }),

  listMembers: () => req('GET', '/api/members'),

  updateMemberRole: (id, role) => req('PUT', `/api/members/${id}/role`, { role }),

  deleteMember: (id) => req('DELETE', `/api/members/${id}`),

  getProfile: () => req('GET', '/api/me'),

  updateProfile: (data) => req('PUT', '/api/me', data),

  listCategories: () => req('GET', '/api/categories'),
}
