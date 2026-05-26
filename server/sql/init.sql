-- SLsec 实验室论坛数据库初始化

CREATE DATABASE IF NOT EXISTS slsec_forum
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci;

USE slsec_forum;

-- 用户表
CREATE TABLE IF NOT EXISTS users (
  id          INT UNSIGNED  AUTO_INCREMENT PRIMARY KEY,
  username    VARCHAR(32)   NOT NULL UNIQUE,
  nickname    VARCHAR(64)   NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  role        ENUM('admin', 'member') NOT NULL DEFAULT 'member',
  avatar      VARCHAR(255),
  created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  deleted_at  DATETIME      NULL DEFAULT NULL,
  INDEX idx_users_username (username),
  INDEX idx_users_deleted_at (deleted_at)
) ENGINE=InnoDB;

-- 分类表
CREATE TABLE IF NOT EXISTS categories (
  id          INT UNSIGNED  AUTO_INCREMENT PRIMARY KEY,
  name        VARCHAR(32)   NOT NULL UNIQUE,
  description VARCHAR(255),
  sort_order  INT           NOT NULL DEFAULT 0
) ENGINE=InnoDB;

-- 文章表
CREATE TABLE IF NOT EXISTS articles (
  id          INT UNSIGNED  AUTO_INCREMENT PRIMARY KEY,
  author_id   INT UNSIGNED  NOT NULL,
  title       VARCHAR(128)  NOT NULL,
  content     MEDIUMTEXT    NOT NULL,
  summary     VARCHAR(512)  NOT NULL DEFAULT '',
  category    VARCHAR(32)   NOT NULL DEFAULT 'general',
  pinned      TINYINT(1)    NOT NULL DEFAULT 0,
  view_count  INT UNSIGNED  NOT NULL DEFAULT 0,
  created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  deleted_at  DATETIME      NULL DEFAULT NULL,
  INDEX idx_articles_author (author_id),
  INDEX idx_articles_created (created_at DESC),
  INDEX idx_articles_category (category),
  INDEX idx_articles_deleted (deleted_at),
  CONSTRAINT fk_articles_author FOREIGN KEY (author_id) REFERENCES users(id)
) ENGINE=InnoDB;

-- 评论表
CREATE TABLE IF NOT EXISTS comments (
  id          INT UNSIGNED  AUTO_INCREMENT PRIMARY KEY,
  article_id  INT UNSIGNED  NOT NULL,
  author_id   INT UNSIGNED  NOT NULL,
  parent_id   INT UNSIGNED  NULL DEFAULT NULL,
  content     TEXT          NOT NULL,
  created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  deleted_at  DATETIME      NULL DEFAULT NULL,
  INDEX idx_comments_article (article_id),
  INDEX idx_comments_author (author_id),
  INDEX idx_comments_parent (parent_id),
  INDEX idx_comments_deleted (deleted_at),
  CONSTRAINT fk_comments_article FOREIGN KEY (article_id) REFERENCES articles(id),
  CONSTRAINT fk_comments_author FOREIGN KEY (author_id) REFERENCES users(id),
  CONSTRAINT fk_comments_parent FOREIGN KEY (parent_id) REFERENCES comments(id)
) ENGINE=InnoDB;

-- 默认分类
INSERT INTO categories (name, description, sort_order) VALUES
  ('general',    '综合讨论', 1),
  ('tech',       '技术分享', 2),
  ('security',   '安全研究', 3),
  ('life',       '生活杂谈', 4);
