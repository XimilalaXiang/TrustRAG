import { existsSync, readFileSync, readdirSync } from 'node:fs'
import path from 'node:path'

const version = process.argv[2] ?? process.env.RELEASE_VERSION
const repo = process.env.GITHUB_REPOSITORY ?? ''
const workspace = process.env.GITHUB_WORKSPACE
  ? path.resolve(process.env.GITHUB_WORKSPACE)
  : process.cwd()

if (!version) {
  fail('Missing release version. Usage: node scripts/generate-release-notes.mjs <version>')
}

const changelogPath = path.join(workspace, 'CHANGELOG.md')

if (!existsSync(changelogPath)) {
  fail(`CHANGELOG.md not found at ${changelogPath}`)
}

const changelog = readFileSync(changelogPath, 'utf8')
const sections = parseChangelogSections(changelog)
const currentIndex = sections.findIndex((section) => section.version === version)

if (currentIndex === -1) {
  const knownVersions = sections.slice(0, 5).map((section) => section.version).join(', ')
  fail(`Version ${version} was not found in CHANGELOG.md. Known versions: ${knownVersions}`)
}

const currentSection = sections[currentIndex]
const previousSection = sections[currentIndex + 1]
const downloadRows = collectDownloadRows(workspace, version)

const lines = [
  `## TrustRAG ${version}`,
  '',
]

if (currentSection.date) {
  lines.push(`> 发布日期 / Release Date: ${currentSection.date}`, '')
}

if (downloadRows.length > 0) {
  lines.push(
    '### 📥 下载 / Downloads',
    '',
    '| 平台 / Platform | 文件 / File | 说明 / Description |',
    '|-----------------|-------------|---------------------|',
  )

  for (const row of downloadRows) {
    lines.push(`| ${row.platform} | ${formatAssetLink(row.filename, repo, version)} | ${row.description} |`)
  }

  lines.push('')
}

lines.push(
  '### 🔄 更新内容 / What\'s Changed',
  '',
  currentSection.body,
  '',
  '### 📖 使用说明 / Usage',
  '',
  '1. 下载对应平台的安装包 / Download the package for your platform',
  '2. 安装并启动 TrustRAG / Install and launch TrustRAG',
  '3. 创建工作区，上传文档，开始提问 / Create a workspace, upload documents, start asking questions',
  '',
  '> ⚠️ macOS 用户注意：应用未经 Apple 签名，首次打开需右键选择"打开"，或在"系统设置 > 隐私与安全性"中允许运行。',
  '> ⚠️ macOS users: The app is not signed by Apple. Right-click and select "Open" for first launch, or allow in "System Settings > Privacy & Security".',
)

if (repo && previousSection) {
  lines.push(
    '',
    '---',
    '',
    `**Full Changelog**: https://github.com/${repo}/compare/v${previousSection.version}...v${version}`,
  )
} else if (repo) {
  lines.push(
    '',
    '---',
    '',
    `**Full Changelog**: https://github.com/${repo}/commits/v${version}`,
  )
}

process.stdout.write(`${normalizeSpacing(lines.join('\n'))}\n`)

function parseChangelogSections(source) {
  const headingPattern = /^## \[(?<version>[^\]]+)\](?: - (?<date>.+))?$/gm
  const matches = [...source.matchAll(headingPattern)]

  return matches.map((match, index) => {
    const start = (match.index ?? 0) + match[0].length
    const end = index + 1 < matches.length ? (matches[index + 1].index ?? source.length) : source.length
    const body = cleanSectionBody(source.slice(start, end))

    return {
      version: match.groups?.version?.trim() ?? '',
      date: match.groups?.date?.trim() ?? '',
      body,
    }
  }).filter((section) => section.version && section.body)
}

function collectDownloadRows(rootDir, releaseVersion) {
  const specs = [
    {
      dir: 'artifacts/trustrag-windows-installer',
      platform: '🪟 Windows',
      pattern: /^TrustRAG-Setup-Windows-x64\.exe$/,
      filename: 'TrustRAG-Setup-Windows-x64.exe',
      description: 'Windows 安装包 / Installer',
    },
    {
      dir: 'artifacts/trustrag-windows-portable',
      platform: '🪟 Windows',
      pattern: /^trustrag-windows-x64-portable\.zip$/,
      filename: 'trustrag-windows-x64-portable.zip',
      description: '便携版 / Portable',
    },
    {
      dir: 'artifacts/trustrag-macos',
      platform: '🍎 macOS',
      pattern: /^trustrag-macos\.tar\.gz$/,
      filename: 'trustrag-macos.tar.gz',
      description: 'macOS 安装包',
    },
    {
      dir: 'artifacts/trustrag-linux-x64',
      platform: '🐧 Linux',
      pattern: /^trustrag-linux-x64\.tar\.gz$/,
      filename: 'trustrag-linux-x64.tar.gz',
      description: 'Linux x64',
    },
    {
      dir: 'artifacts/trustrag-android',
      platform: '🤖 Android',
      pattern: /^app-release\.apk$/,
      filename: 'app-release.apk',
      description: 'Android APK',
    },
    {
      dir: 'artifacts/trustrag-ios',
      platform: '📱 iOS',
      pattern: /^trustrag-ios\.tar\.gz$/,
      filename: 'trustrag-ios.tar.gz',
      description: 'iOS App (unsigned)',
    },
    {
      dir: 'artifacts/trustrag-web',
      platform: '🌐 Web',
      pattern: /^trustrag-web\.tar\.gz$/,
      filename: 'trustrag-web.tar.gz',
      description: 'Web 静态资源 / Static files',
    },
  ]

  return specs.flatMap((spec) => {
    const dirPath = path.join(rootDir, spec.dir)

    if (!existsSync(dirPath)) {
      return [{
        platform: spec.platform,
        filename: spec.filename,
        description: spec.description,
      }]
    }

    const filename = readdirSync(dirPath).find((entry) => spec.pattern.test(entry))

    if (!filename) {
      return []
    }

    return [{
      platform: spec.platform,
      filename,
      description: spec.description,
    }]
  })
}

function formatAssetLink(filename, repoName, releaseVersion) {
  if (!repoName) {
    return `\`${filename}\``
  }

  const encodedFilename = encodeURIComponent(filename)
  const url = `https://github.com/${repoName}/releases/download/v${releaseVersion}/${encodedFilename}`
  return `[${filename}](${url})`
}

function normalizeSpacing(markdown) {
  return markdown
    .replace(/\r\n/g, '\n')
    .replace(/\n{3,}/g, '\n\n')
    .trimEnd()
}

function cleanSectionBody(body) {
  return body
    .replace(/\r\n/g, '\n')
    .replace(/\n---\s*$/u, '')
    .trim()
}

function fail(message) {
  console.error(message)
  process.exit(1)
}
