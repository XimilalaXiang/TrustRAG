# UI 组件策略（2026-05-14）

## 原则

不从头设计 UI 组件，使用社区成熟组件库，只自定义差异化交互。

## 推荐组合

| 层次 | 组件来源 | 说明 |
|------|---------|------|
| 基础组件 | Flutter Material 3（内置） | 按钮、卡片、输入框、导航等 |
| PDF 阅读器 | Syncfusion PDF Viewer | 页码跳转、文本高亮、搜索 |
| 数据表格 | Syncfusion DataGrid 或 om_data_grid | 资料库列表、复核列表 |
| Dashboard 布局 | Focus UI Kit 或自建 | 首页仪表盘 |
| 图表 | fl_chart 或 Syncfusion Charts | 统计图表 |
| Markdown 渲染 | flutter_markdown | AI 回答展示 |
| 主题系统 | Material 3 ThemeData | 统一颜色/字体/间距 |

## 需要自定义的组件

1. 引用来源卡片样式
2. 复核状态徽章
3. AI 对话中的引用标记交互

## 参考项目

- Focus Flutter UI Kit: github.com/maxlam79/focus_flutter_ui_kit (MIT)
- Syncfusion: 社区版免费（个人/小团队 <$1M 收入）
