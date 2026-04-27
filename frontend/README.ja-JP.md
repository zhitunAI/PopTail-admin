<div align="center">
  <a href="https://github.com/zhitunAI/PopTail-admin">
    <img alt="VbenAdmin Logo" width="215" src="https://unpkg.com/@vbenjs/static-source@0.1.7/source/logo-v1.webp">
  </a>
  <br>
  <br>

[![license](https://img.shields.io/github/license/zhitunAI/PopTail-admin.svg)](LICENSE)

  <h1>PopTail-admin</h1>
</div>

[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=zhitunAI_PopTail-admin&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=zhitunAI_PopTail-admin) ![codeql](https://github.com/zhitunAI/PopTail-admin/actions/workflows/codeql.yml/badge.svg) ![build](https://github.com/zhitunAI/PopTail-admin/actions/workflows/build.yml/badge.svg) ![ci](https://github.com/zhitunAI/PopTail-admin/actions/workflows/ci.yml/badge.svg) ![deploy](https://github.com/zhitunAI/PopTail-admin/actions/workflows/deploy.yml/badge.svg)

**日本語** | [English](./README.md) | [中文](./README.zh-CN.md)

## 紹介

PopTail-adminは、最新の`vue3`、`vite`、`TypeScript`などの主流技術を使用して開発された、無料でオープンソースの中・後端テンプレートです。すぐに使える中・後端のフロントエンドソリューションとして、学習の参考にもなります。

このプロジェクトは `vue-vben-admin` フレームワークをベースにしています：

- [https://github.com/vbenjs/vue-vben-admin](https://github.com/vbenjs/vue-vben-admin)

## アップグレード通知

これは最新の `vue-vben-admin` フレームワーク系統をベースにした PopTail-admin 実装です。

## 特徴

- **最新技術スタック**：Vue 3やViteなどの最先端フロントエンド技術で開発
- **TypeScript**：アプリケーション規模のJavaScriptのための言語
- **テーマ**：複数のテーマカラーが利用可能で、カスタマイズオプションも豊富
- **国際化**：完全な内蔵国際化サポート
- **権限管理**：動的ルートベースの権限生成ソリューションを内蔵

## プレビュー

- [PopTail-admin リポジトリ](https://github.com/zhitunAI/PopTail-admin)

テストアカウント：vben/123456

<div align="center">
  <img alt="VbenAdmin Logo" width="100%" src="https://anncwb.github.io/anncwb/images/preview1.png">
  <img alt="VbenAdmin Logo" width="100%" src="https://anncwb.github.io/anncwb/images/preview2.png">
  <img alt="VbenAdmin Logo" width="100%" src="https://anncwb.github.io/anncwb/images/preview3.png">
</div>

### Gitpodを使用

Gitpod（GitHub用の無料オンライン開発環境）でプロジェクトを開き、すぐにコーディングを開始します。

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/zhitunAI/PopTail-admin)

## ドキュメント

[プロジェクトリポジトリ](https://github.com/zhitunAI/PopTail-admin)

## インストールと使用

1. プロジェクトコードを取得

```bash
git clone https://github.com/zhitunAI/PopTail-admin.git
```

2. 依存関係のインストール

```bash
cd PopTail-admin/frontend
npm i -g corepack
pnpm install
```

3. 実行

```bash
pnpm dev
```

4. ビルド

```bash
pnpm build
```

## 変更ログ

[CHANGELOG](https://github.com/zhitunAI/PopTail-admin/releases)

## 貢献方法

ご参加をお待ちしております！[Issueを提出](https://github.com/zhitunAI/PopTail-admin/issues/new/choose)するか、Pull Requestを送信してください。

**Pull Request プロセス：**

1. コードをフォーク
2. 自分のブランチを作成：`git checkout -b feat/xxxx`
3. 変更をコミット：`git commit -am 'feat(function): add xxxxx'`
4. ブランチをプッシュ：`git push origin feat/xxxx`
5. `pull request`を送信

## Git貢献提出規則

参考 [vue](https://github.com/vuejs/vue/blob/dev/.github/COMMIT_CONVENTION.md) 規則 ([Angular](https://github.com/conventional-changelog/conventional-changelog/tree/master/packages/conventional-changelog-angular))

- `feat` 新機能の追加
- `fix` 問題/バグの修正
- `style` コードスタイルに関連し、実行結果に影響しない
- `perf` 最適化/パフォーマンス向上
- `refactor` リファクタリング
- `revert` 変更の取り消し
- `test` テスト関連
- `docs` ドキュメント/注釈
- `chore` 依存関係の更新/スキャフォールディング設定の変更など
- `ci` 継続的インテグレーション
- `types` 型定義ファイルの変更

## ブラウザサポート

ローカル開発には `Chrome 80+` ブラウザを推奨します

モダンブラウザをサポートし、IEはサポートしません

| [<img src="https://raw.githubusercontent.com/alrra/browser-logos/master/src/edge/edge_48x48.png" alt="Edge" width="24px" height="24px" />](http://godban.github.io/browsers-support-badges/)</br>Edge | [<img src="https://raw.githubusercontent.com/alrra/browser-logos/master/src/firefox/firefox_48x48.png" alt="Firefox" width="24px" height="24px" />](http://godban.github.io/browsers-support-badges/)</br>Firefox | [<img src="https://raw.githubusercontent.com/alrra/browser-logos/master/src/chrome/chrome_48x48.png" alt="Chrome" width="24px" height="24px" />](http://godban.github.io/browsers-support-badges/)</br>Chrome | [<img src="https://raw.githubusercontent.com/alrra/browser-logos/master/src/safari/safari_48x48.png" alt="Safari" width="24px" height="24px" />](http://godban.github.io/browsers-support-badges/)</br>Safari |
| :-: | :-: | :-: | :-: |
| 最新2バージョン | 最新2バージョン | 最新2バージョン | 最新2バージョン |

## メンテナー

[@zhitunAI](https://github.com/zhitunAI)

## スター歴史

[![Star History Chart](https://api.star-history.com/svg?repos=vbenjs/vue-vben-admin&type=Date)](https://star-history.com/#vbenjs/vue-vben-admin&Date)

## 寄付

このプロジェクトが役に立つと思われた場合、作者にコーヒーを一杯おごってサポートを示すことができます！

![donate](https://unpkg.com/@vbenjs/static-source@0.1.7/source/sponsor.png)

<a style="display: block;width: 100px;height: 50px;line-height: 50px; color: #fff;text-align: center; background: #408aed;border-radius: 4px;" href="https://www.paypal.com/paypalme/cvvben">Paypal Me</a>

## 貢献者

<a href="https://openomy.app/github/vbenjs/vue-vben-admin" target="_blank" style="display: block; width: 100%;" align="center">
  <img src="https://openomy.app/svg?repo=vbenjs/vue-vben-admin&chart=bubble&latestMonth=3" target="_blank" alt="Contribution Leaderboard" style="display: block; width: 100%;" />
 </a>

<a href="https://github.com/zhitunAI/PopTail-admin/graphs/contributors">
  <img alt="Contributors" src="https://contrib.rocks/image?repo=vbenjs/vue-vben-admin" />
</a>

## Discord

- [GitHub Discussions](https://github.com/zhitunAI/PopTail-admin/discussions)

## ライセンス

[MIT © Vben-2020](./LICENSE)
