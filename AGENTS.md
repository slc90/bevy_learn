# AGENTS.md

## 项目介绍

这是一个用于学习 Bevy 和 Rust 游戏开发的 workspace 项目。项目以 Cargo workspace 管理代码，当前主要代码入口放在 `crates` 目录下，并配套维护学习资料、需求记录、开发规则和常用命令，方便后续按模块逐步扩展 Bevy 示例与 ECS 实验代码。

## 当前层级结构

- `.gitignore`：Git 忽略规则，主要用于排除构建产物和生成内容。

- `AGENTS.md`：面向自动化开发助手和协作者的项目说明文件。

- `Cargo.lock`：Cargo 依赖锁定文件，用于固定 workspace 当前解析出的依赖版本。

- `Cargo.toml`：Cargo workspace 根配置，声明 workspace 成员和共享配置。

- `bevy_book`：Bevy 学习笔记和 mdBook 资料目录。

- `crates`：Rust workspace 成员 crate 的存放目录。

- `docs`：项目文档目录，用于存放设计说明、方案记录等文档。

- `requirements`：需求文档目录，用于记录功能需求、学习目标或任务描述。

- `rules`：项目规范目录，用于维护代码规范、开发规范、提交规范和常用命令。
