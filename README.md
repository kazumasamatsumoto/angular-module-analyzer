# Angular Module Analyzer

AngularプロジェクトのCore/Shared/Featureモジュール構造を解析し、アーキテクチャの健全性をチェックするRustツールです。

## 機能

### 🔍 モジュール発見と分類
- `.module.ts`ファイルを自動検出
- Core/Shared/Feature/Unknownに自動分類
- パス構造による分類ロジック

### 📊 依存関係分析
- モジュール間の依存関係を抽出
- 依存関係違反の検出
  - CoreがFeatureに依存
  - SharedがFeatureに依存
  - Feature間の直接依存
- 循環依存の検出

### 📈 メトリクス計算
- モジュール数の統計
- 平均依存関係数
- 結合度（Coupling Factor）
- 依存関係の深さ

### 🎨 可視化
- DOT形式の依存関係グラフ生成
- Graphvizでの可視化対応

## インストール

```bash
# プロジェクトのクローン
git clone <repository-url>
cd angular-module-analyzer

# ビルド
cargo build --release
```

## 使用方法

### 基本的な解析

```bash
# コンソール出力で解析結果を表示
./target/release/analyze analyze -p /path/to/angular/project

# JSON形式で出力
./target/release/analyze analyze -p /path/to/angular/project -o json
```

### 依存関係グラフの生成

```bash
# DOTファイルの生成
./target/release/analyze graph -p /path/to/angular/project -o deps.dot

# Graphvizで画像生成
dot -Tpng deps.dot -o dependency-graph.png
```

## 出力例

### コンソール出力
```
=== Angular Module Analysis Report ===

📊 Architecture Metrics
Total Modules: 12
Core Modules: 2
Shared Modules: 3
Feature Modules: 7
Average Dependencies per Module: 3.50
Coupling Factor: 0.15

⚠️  Dependency Violations
  CoreModule -> UserFeatureModule: Core module depends on Feature module
  SharedModule -> OrderFeatureModule: Shared module depends on Feature module

📦 Modules by Type
  Core:
    - CoreModule (2 dependencies)
    - AuthModule (1 dependencies)
  
  Shared:
    - SharedModule (4 dependencies)
    - UIModule (2 dependencies)
    - UtilsModule (1 dependencies)
  
  Feature:
    - UserFeatureModule (5 dependencies)
    - OrderFeatureModule (3 dependencies)
    - ProductFeatureModule (4 dependencies)
```

### JSON出力
```json
{
  "modules": [
    {
      "path": "/src/app/core/core.module.ts",
      "name": "CoreModule",
      "module_type": "Core",
      "imports": ["CommonModule", "HttpClientModule"],
      "exports": ["AuthService"],
      "providers": ["AuthService", "ApiService"],
      "declarations": [],
      "dependencies": ["@shared/ui", "@shared/utils"]
    }
  ],
  "dependency_violations": [
    {
      "from_module": "CoreModule",
      "to_module": "UserFeatureModule",
      "violation_type": "CoreDependsOnFeature",
      "description": "Core module depends on Feature module"
    }
  ],
  "circular_dependencies": [],
  "metrics": {
    "total_modules": 12,
    "core_modules": 2,
    "shared_modules": 3,
    "feature_modules": 7,
    "average_dependencies_per_module": 3.5,
    "max_dependency_depth": 4,
    "coupling_factor": 0.15
  }
}
```

## アーキテクチャルール

このツールは以下のAngularアーキテクチャルールをチェックします：

### ✅ 良い依存関係
- Feature → Shared
- Feature → Core
- Shared → Core

### ❌ 避けるべき依存関係
- Core → Feature
- Shared → Feature
- Feature → Feature（直接依存）

### 📁 ディレクトリ構造の想定
```
src/app/
├── core/           # コアモジュール
├── shared/         # 共有モジュール
├── features/       # フィーチャーモジュール
│   ├── user/
│   ├── order/
│   └── product/
```

## 拡張方法

### カスタム分類ロジック
`determine_module_type`メソッドを修正して、プロジェクト固有の分類ロジックを追加できます。

### 新しいメトリクス
`calculate_metrics`メソッドに新しいメトリクスを追加できます。

### カスタムルール
`check_dependency_violations`メソッドに新しいアーキテクチャルールを追加できます。

## 依存関係

- `clap`: コマンドライン引数解析
- `serde`: JSON シリアライゼーション
- `walkdir`: ディレクトリ走査
- `regex`: 正規表現
- `petgraph`: グラフ操作
- `colored`: カラー出力

## 今後の改善点

- [ ] TypeScript AST解析の実装
- [ ] より精密な循環依存検出
- [ ] インタラクティブなWeb UI
- [ ] CI/CD統合サポート
- [ ] カスタムルール設定ファイル
- [ ] パフォーマンス最適化

## ライセンス

MIT License