#!/usr/bin/env node

import * as fs from 'fs-extra';
import * as path from 'path';
import inquirer from 'inquirer';
import chalk from 'chalk';
import ora from 'ora';
import { glob } from 'glob';

interface SkeletonOptions {
  targetPath: string;
  projectName: string;
  includeDocs: boolean;
  includeScripts: boolean;
  includePrompts: boolean;
  includeInfrastructure: boolean;
  customCursorRules: boolean;
}

class SkeletonGenerator {
  private sourceDir: string;
  private options!: SkeletonOptions;

  constructor() {
    this.sourceDir = path.resolve(__dirname, '..');
  }

  async run(): Promise<void> {
    console.log(chalk.blue.bold('🏗️  wezterm-parallel - スケルトン生成ツール'));
    console.log(chalk.gray('新しいプロジェクトのスケルトンを生成します\n'));

    try {
      await this.promptOptions();
      await this.validateTargetPath();
      await this.generateSkeleton();
      await this.postProcess();
      
      console.log(chalk.green.bold('\n✅ スケルトンの生成が完了しました！'));
      this.printNextSteps();
    } catch (error) {
      console.error(chalk.red.bold('\n❌ エラーが発生しました:'), error);
      process.exit(1);
    }
  }

  private async promptOptions(): Promise<void> {
    const answers = await inquirer.prompt([
      {
        type: 'input',
        name: 'targetPath',
        message: '生成先のパスを入力してください:',
        default: './my-new-project',
        validate: (input: string) => {
          if (!input.trim()) {
            return 'パスを入力してください';
          }
          return true;
        }
      },
      {
        type: 'input',
        name: 'projectName',
        message: 'プロジェクト名を入力してください:',
        default: 'my-new-project',
        validate: (input: string) => {
          if (!input.trim()) {
            return 'プロジェクト名を入力してください';
          }
          if (!/^[a-zA-Z0-9-_]+$/.test(input)) {
            return 'プロジェクト名は英数字、ハイフン、アンダースコアのみ使用可能です';
          }
          return true;
        }
      },
      {
        type: 'confirm',
        name: 'includeDocs',
        message: 'ドキュメントファイルを含めますか？',
        default: true
      },
      {
        type: 'confirm',
        name: 'includeScripts',
        message: 'スクリプトファイルを含めますか？',
        default: true
      },
      {
        type: 'confirm',
        name: 'includePrompts',
        message: 'プロンプトファイルを含めますか？',
        default: true
      },
      {
        type: 'confirm',
        name: 'includeInfrastructure',
        message: 'インフラストラクチャファイルを含めますか？',
        default: false
      },
      {
        type: 'confirm',
        name: 'customCursorRules',
        message: 'プロジェクト固有の .cursorrules を生成しますか？',
        default: true
      }
    ]);

    this.options = answers as SkeletonOptions;
  }

  private async validateTargetPath(): Promise<void> {
    const targetPath = path.resolve(this.options.targetPath);
    
    if (await fs.pathExists(targetPath)) {
      const { overwrite } = await inquirer.prompt([
        {
          type: 'confirm',
          name: 'overwrite',
          message: `ディレクトリ "${targetPath}" は既に存在します。上書きしますか？`,
          default: false
        }
      ]);

      if (!overwrite) {
        throw new Error('ユーザーによってキャンセルされました');
      }

      await fs.remove(targetPath);
    }
  }

  private async generateSkeleton(): Promise<void> {
    const spinner = ora('スケルトンを生成中...').start();
    const targetPath = path.resolve(this.options.targetPath);

    try {
      // 基本ディレクトリ構造を作成
      await fs.ensureDir(targetPath);

      // コピーするファイルとディレクトリのリスト
      const copyItems = [
        'README.md',
        'package.json',
        'package-lock.json',
        '.gitignore',
        'CLAUDE.md',
        'CONTRIBUTING.md',
        'CUSTOMIZATION_GUIDE.md',
        'DEVELOPMENT_ROADMAP.md',
        'FEATURE_SUMMARY.md',
        'PROGRESS.md',
        'PROJECT_STRUCTURE.md',
        'PROMPT.md'
      ];

      // オプションに基づいてディレクトリを追加
      if (this.options.includeDocs) {
        copyItems.push('docs');
      }
      if (this.options.includeScripts) {
        copyItems.push('scripts');
      }
      if (this.options.includePrompts) {
        copyItems.push('prompts');
      }
      if (this.options.includeInfrastructure) {
        copyItems.push('infrastructure');
      }

      // ファイルとディレクトリをコピー
      for (const item of copyItems) {
        const sourcePath = path.join(this.sourceDir, item);
        const targetItemPath = path.join(targetPath, item);

        if (await fs.pathExists(sourcePath)) {
          await fs.copy(sourcePath, targetItemPath);
        }
      }

      // .cursorrules を生成
      if (this.options.customCursorRules) {
        await this.generateCursorRules(targetPath);
      }

      // package.json を更新
      await this.updatePackageJson(targetPath);

      spinner.succeed('スケルトンの生成が完了しました');
    } catch (error) {
      spinner.fail('スケルトンの生成に失敗しました');
      throw error;
    }
  }

  private async generateCursorRules(targetPath: string): Promise<void> {
    const cursorRulesContent = `# Cursor Rules - 日本語コミュニケーション設定

## 会話ガイドライン
- 常に日本語で会話する

## 開発哲学

### テスト駆動開発（TDD）
- 原則としてテスト駆動開発（TDD）で進める
- 期待される入出力に基づき、まずテストを作成する
- 実装コードは書かず、テストのみを用意する
- テストを実行し、失敗を確認する
- テストが正しいことを確認できた段階でコミットする
- その後、テストをパスさせる実装を進める
- 実装中はテストを変更せず、コードを修正し続ける
- すべてのテストが通過するまで繰り返す

## 言語設定
- 常に日本語でコミュニケーションを行ってください
- コードコメントも日本語で記述してください
- エラーメッセージやログの説明も日本語で行ってください

## コーディングスタイル
- 変数名や関数名は英語で記述（プログラミングの慣例に従う）
- コメント、ドキュメント、READMEは日本語で記述
- コミットメッセージは日本語で記述

## コミュニケーション
- 技術的な説明は分かりやすい日本語で行ってください
- 専門用語を使用する場合は、必要に応じて説明を加えてください
- 質問や確認は日本語で行ってください

## プロジェクト固有の設定
- このプロジェクトは ${this.options.projectName} です
- 開発環境のセットアップや設定に関する質問は日本語で対応してください
- ドキュメントの作成や更新も日本語で行ってください

## ファイル命名規則
- 設定ファイルやドキュメントファイルは日本語名も可
- ソースコードファイルは英語名で統一
- ディレクトリ名は英語で統一

## エラーハンドリング
- エラーメッセージの説明は日本語で行ってください
- デバッグ情報も日本語で提供してください
- トラブルシューティングの手順も日本語で説明してください
`;

    await fs.writeFile(path.join(targetPath, '.cursorrules'), cursorRulesContent);
  }

  private async updatePackageJson(targetPath: string): Promise<void> {
    const packageJsonPath = path.join(targetPath, 'package.json');
    const packageJson = await fs.readJson(packageJsonPath);

    // プロジェクト名を更新
    packageJson.name = this.options.projectName;
    packageJson.description = `${this.options.projectName} - wezterm-parallel Project`;

    // リポジトリ情報をクリア
    delete packageJson.repository;
    delete packageJson.bugs;
    delete packageJson.homepage;

    await fs.writeJson(packageJsonPath, packageJson, { spaces: 2 });
  }

  private async postProcess(): Promise<void> {
    const targetPath = path.resolve(this.options.targetPath);
    
    // .git ディレクトリを削除（新しいリポジトリとして初期化するため）
    const gitPath = path.join(targetPath, '.git');
    if (await fs.pathExists(gitPath)) {
      await fs.remove(gitPath);
    }

    // node_modules を削除（新しくインストールするため）
    const nodeModulesPath = path.join(targetPath, 'node_modules');
    if (await fs.pathExists(nodeModulesPath)) {
      await fs.remove(nodeModulesPath);
    }
  }

  private printNextSteps(): void {
    const targetPath = path.resolve(this.options.targetPath);
    
    console.log(chalk.cyan.bold('\n📋 次のステップ:'));
    console.log(chalk.white(`1. プロジェクトディレクトリに移動:`));
    console.log(chalk.gray(`   cd ${targetPath}`));
    console.log(chalk.white(`2. 依存関係をインストール:`));
    console.log(chalk.gray(`   npm install`));
    console.log(chalk.white(`3. 開発を開始:`));
    console.log(chalk.gray(`   npm run setup`));
    console.log(chalk.white(`4. Git リポジトリを初期化:`));
    console.log(chalk.gray(`   git init`));
    console.log(chalk.gray(`   git add .`));
    console.log(chalk.gray(`   git commit -m "Initial commit"`));
    console.log(chalk.cyan.bold('\n🎉 新しいプロジェクトの準備が完了しました！'));
  }
}

// メイン実行
if (require.main === module) {
  const generator = new SkeletonGenerator();
  generator.run().catch(console.error);
}

export default SkeletonGenerator; 