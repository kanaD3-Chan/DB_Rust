# SLsec Forum

SLsec 实验室论坛系统，基于 Rust 构建，支持终端 TUI、桌面 GUI（Tauri + React）多端访问。

## 项目结构

```
├── server/          # Axum API 服务器
├── client/          # Ratatui 终端 TUI 客户端
├── tauri-client/    # Tauri + React 桌面客户端
├── docker-compose.yml
└── Dockerfile.server
```

## 技术栈

**后端：** Axum + SQLx (MySQL) + JWT + bcrypt + WebSocket

**数据库：** MySQL 8.0

**客户端：**
| 客户端 | 技术 |
|--------|------|
| 终端 TUI | Ratatui + Crossterm + reqwest |
| 桌面 GUI | Tauri v2 + React 19 + Vite |

## 功能

- 用户注册/登录（JWT 无状态认证）
- 文章发布、编辑、软删除（Markdown 支持）
- 文章分类筛选与搜索
- 分页浏览
- 嵌套评论回复
- WebSocket 实时通知
- 成员管理（admin 角色）
- 个人资料修改
- 多主题切换（桌面客户端）

## 快速开始

### Docker Compose（推荐）

```bash
# 构建服务器二进制
cargo build --release

# 启动 MySQL + 服务器
docker-compose up -d
```

服务默认监听 `http://localhost:3000`。

### 手动运行

1. 启动 MySQL 并执行 `server/sql/init.sql`
2. 配置环境变量（或创建 `server/.env`）：

```env
DATABASE_URL=mysql://root:password@localhost:3306/slsec_forum
JWT_SECRET=your-secret-key
PORT=3000
```

3. 启动服务器：

```bash
cargo run -p forum-server
```

4. 启动 TUI 客户端：

```bash
cargo run -p forum-client
```

5. 启动桌面客户端：

```bash
cd tauri-client
npm install
npm run tauri dev
```

## 数据库

首次运行需执行 `server/sql/init.sql` 初始化表结构。Docker Compose 部署会自动完成此步骤。

- 第一个注册的用户自动成为管理员
- 删除操作为软删除（设置 `deleted_at` 字段）

## API 概览

### 公开接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/register` | 用户注册 |
| POST | `/api/login` | 用户登录 |
| GET  | `/api/articles` | 文章列表（分页、分类、搜索） |
| GET  | `/api/articles/{id}` | 文章详情 |
| GET  | `/api/articles/{id}/comments` | 评论列表 |
| GET  | `/api/categories` | 分类列表 |
| GET  | `/ws` | WebSocket 通知 |

### 需认证（Header: `Authorization: Bearer <token>`）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/articles` | 创建文章 |
| PUT  | `/api/articles/{id}` | 编辑文章（作者/admin） |
| DELETE | `/api/articles/{id}` | 删除文章（作者/admin） |
| POST | `/api/articles/{id}/comments` | 发表评论 |
| GET  | `/api/me` | 获取个人资料 |
| PUT  | `/api/me` | 更新个人资料 |
| GET  | `/api/members` | 成员列表（admin） |
| PUT  | `/api/members/{id}/role` | 修改角色（admin） |
| DELETE | `/api/members/{id}` | 删除成员（admin） |

所有接口统一返回 `{ success: bool, data?: T, error?: string }` 格式。

## 许可证

MIT
