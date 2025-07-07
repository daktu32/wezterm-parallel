#!/usr/bin/env python3
"""
WezTerm Parallel パフォーマンス分析ツール
ベンチマーク結果の可視化と詳細分析を行います。
"""

import json
import sys
import argparse
import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
from pathlib import Path
from datetime import datetime
import seaborn as sns

# スタイル設定
plt.style.use('dark_background')
sns.set_palette("husl")

class PerformanceAnalyzer:
    def __init__(self, results_file):
        """分析器を初期化"""
        self.results_file = Path(results_file)
        self.results = self._load_results()
        self.output_dir = self.results_file.parent / "analysis"
        self.output_dir.mkdir(exist_ok=True)
    
    def _load_results(self):
        """ベンチマーク結果を読み込み"""
        try:
            with open(self.results_file) as f:
                return json.load(f)
        except FileNotFoundError:
            print(f"❌ 結果ファイルが見つかりません: {self.results_file}")
            sys.exit(1)
        except json.JSONDecodeError as e:
            print(f"❌ JSONファイルの解析に失敗: {e}")
            sys.exit(1)
    
    def analyze_startup_time(self):
        """起動時間の分析とグラフ作成"""
        startup_data = self.results['benchmarks']['startup_time']
        valid_times = [t for t in startup_data if t is not None]
        
        if not valid_times:
            print("⚠️ 起動時間データがありません")
            return
        
        print("📊 起動時間分析")
        print(f"   平均: {np.mean(valid_times):.3f}秒")
        print(f"   中央値: {np.median(valid_times):.3f}秒")
        print(f"   標準偏差: {np.std(valid_times):.3f}秒")
        print(f"   範囲: {np.min(valid_times):.3f}秒 - {np.max(valid_times):.3f}秒")
        
        # グラフ作成
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
        fig.suptitle('起動時間分析', fontsize=16, fontweight='bold')
        
        # 時系列プロット
        ax1.plot(range(1, len(valid_times) + 1), valid_times, 
                marker='o', linewidth=2, markersize=8, color='#00ff00')
        ax1.axhline(y=np.mean(valid_times), color='#ff6b35', 
                   linestyle='--', linewidth=2, label=f'平均: {np.mean(valid_times):.3f}s')
        ax1.set_xlabel('実行回数')
        ax1.set_ylabel('起動時間 (秒)')
        ax1.set_title('起動時間の推移')
        ax1.grid(True, alpha=0.3)
        ax1.legend()
        
        # ヒストグラム
        ax2.hist(valid_times, bins=10, edgecolor='white', alpha=0.7, color='#89b4fa')
        ax2.axvline(x=np.mean(valid_times), color='#ff6b35', 
                   linestyle='--', linewidth=2, label=f'平均: {np.mean(valid_times):.3f}s')
        ax2.set_xlabel('起動時間 (秒)')
        ax2.set_ylabel('頻度')
        ax2.set_title('起動時間の分布')
        ax2.legend()
        
        plt.tight_layout()
        output_file = self.output_dir / "startup_time_analysis.png"
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        print(f"   グラフ保存: {output_file}")
        plt.close()
    
    def analyze_memory_usage(self):
        """メモリ使用量の分析とグラフ作成"""
        memory_data = self.results['benchmarks']['memory_usage']
        
        if not memory_data or 'samples' not in memory_data:
            print("⚠️ メモリ使用量データがありません")
            return
        
        samples = memory_data['samples']
        
        print("📊 メモリ使用量分析")
        print(f"   平均: {memory_data['avg_mb']:.2f}MB")
        print(f"   最小: {memory_data['min_mb']:.2f}MB")
        print(f"   最大: {memory_data['max_mb']:.2f}MB")
        print(f"   変動係数: {(np.std(samples) / np.mean(samples) * 100):.2f}%")
        
        # グラフ作成
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(15, 10))
        fig.suptitle('メモリ使用量分析', fontsize=16, fontweight='bold')
        
        # 時系列プロット
        time_points = np.arange(len(samples))
        ax1.plot(time_points, samples, linewidth=2, color='#a6e3a1')
        ax1.fill_between(time_points, samples, alpha=0.3, color='#a6e3a1')
        ax1.axhline(y=np.mean(samples), color='#ff6b35', 
                   linestyle='--', linewidth=2, label=f'平均: {np.mean(samples):.2f}MB')
        ax1.set_xlabel('時間 (秒)')
        ax1.set_ylabel('メモリ使用量 (MB)')
        ax1.set_title('メモリ使用量の時系列変化')
        ax1.grid(True, alpha=0.3)
        ax1.legend()
        
        # 移動平均とトレンド
        window_size = min(10, len(samples) // 4)
        if window_size > 1:
            moving_avg = pd.Series(samples).rolling(window=window_size).mean()
            ax2.plot(time_points, samples, alpha=0.5, color='#a6e3a1', label='実測値')
            ax2.plot(time_points, moving_avg, linewidth=3, color='#89b4fa', 
                    label=f'移動平均 ({window_size}秒)')
            
            # トレンド分析
            z = np.polyfit(time_points, samples, 1)
            trend_line = np.poly1d(z)
            ax2.plot(time_points, trend_line(time_points), 
                    linewidth=2, linestyle=':', color='#f9e2af', 
                    label=f'トレンド: {z[0]:.3f}MB/s')
        
        ax2.set_xlabel('時間 (秒)')
        ax2.set_ylabel('メモリ使用量 (MB)')
        ax2.set_title('メモリ使用量のトレンド分析')
        ax2.grid(True, alpha=0.3)
        ax2.legend()
        
        plt.tight_layout()
        output_file = self.output_dir / "memory_usage_analysis.png"
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        print(f"   グラフ保存: {output_file}")
        plt.close()
    
    def analyze_api_response(self):
        """API応答性能の分析とグラフ作成"""
        api_data = self.results['benchmarks']['api_response']
        
        if not api_data:
            print("⚠️ API応答データがありません")
            return
        
        print("📊 API応答性能分析")
        
        # データ整理
        endpoints = []
        response_times = []
        success_rates = []
        
        for item in api_data:
            if 'error' not in item:
                endpoints.append(item['endpoint'].split('/')[-1])  # パスの最後の部分
                response_times.append(item['avg_response_time'] * 1000)  # ミリ秒変換
                success_rates.append(item['success_rate'] * 100)  # パーセント変換
                print(f"   {item['endpoint']}: {item['avg_response_time']*1000:.2f}ms (成功率: {item['success_rate']*100:.1f}%)")
        
        if not endpoints:
            print("⚠️ 有効なAPI応答データがありません")
            return
        
        # グラフ作成
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
        fig.suptitle('API応答性能分析', fontsize=16, fontweight='bold')
        
        # 応答時間バーチャート
        bars1 = ax1.bar(endpoints, response_times, color='#f38ba8', alpha=0.8)
        ax1.set_ylabel('平均応答時間 (ms)')
        ax1.set_title('エンドポイント別応答時間')
        ax1.tick_params(axis='x', rotation=45)
        
        # 値をバーの上に表示
        for bar, time in zip(bars1, response_times):
            ax1.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.5,
                    f'{time:.1f}ms', ha='center', va='bottom')
        
        # 成功率バーチャート
        bars2 = ax2.bar(endpoints, success_rates, color='#94e2d5', alpha=0.8)
        ax2.set_ylabel('成功率 (%)')
        ax2.set_title('エンドポイント別成功率')
        ax2.set_ylim(0, 100)
        ax2.tick_params(axis='x', rotation=45)
        
        # 値をバーの上に表示
        for bar, rate in zip(bars2, success_rates):
            ax2.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1,
                    f'{rate:.1f}%', ha='center', va='bottom')
        
        plt.tight_layout()
        output_file = self.output_dir / "api_response_analysis.png"
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        print(f"   グラフ保存: {output_file}")
        plt.close()
    
    def analyze_stress_test(self):
        """ストレステストの分析とグラフ作成"""
        stress_data = self.results['benchmarks']['stress_test']
        
        if not stress_data:
            print("⚠️ ストレステストデータがありません")
            return
        
        print("📊 ストレステスト分析")
        
        # ワークスペース作成性能
        if 'workspace_creation' in stress_data:
            ws_data = stress_data['workspace_creation']
            print(f"   ワークスペース作成:")
            print(f"     成功率: {ws_data['success_count']}/{ws_data['target_count']} ({ws_data['success_count']/ws_data['target_count']*100:.1f}%)")
            print(f"     スループット: {ws_data['throughput_ops_per_second']:.2f} ops/sec")
        
        # API負荷テスト
        if 'api_load_test' in stress_data:
            api_data = stress_data['api_load_test']
            print(f"   API負荷テスト:")
            print(f"     成功率: {api_data['success_rate']*100:.1f}%")
            print(f"     継続時間: {api_data['duration_seconds']:.1f}秒")
            if api_data.get('total_requests', 0) > 0:
                print(f"     平均スループット: {api_data['total_requests']/api_data['duration_seconds']:.1f} req/sec")
        
        # グラフ作成
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
        fig.suptitle('ストレステスト結果', fontsize=16, fontweight='bold')
        
        # ワークスペース作成結果
        if 'workspace_creation' in stress_data:
            ws_data = stress_data['workspace_creation']
            success_count = ws_data['success_count']
            failure_count = ws_data['target_count'] - success_count
            
            ax1.pie([success_count, failure_count], 
                   labels=['成功', '失敗'], 
                   colors=['#a6e3a1', '#f38ba8'],
                   autopct='%1.1f%%',
                   startangle=90)
            ax1.set_title(f'ワークスペース作成結果\n(スループット: {ws_data["throughput_ops_per_second"]:.2f} ops/sec)')
        
        # API負荷テスト結果
        if 'api_load_test' in stress_data:
            api_data = stress_data['api_load_test']
            success_rate = api_data['success_rate'] * 100
            failure_rate = 100 - success_rate
            
            ax2.pie([success_rate, failure_rate], 
                   labels=['成功', '失敗'], 
                   colors=['#94e2d5', '#f9e2af'],
                   autopct='%1.1f%%',
                   startangle=90)
            ax2.set_title(f'API負荷テスト結果\n(継続時間: {api_data["duration_seconds"]:.1f}秒)')
        
        plt.tight_layout()
        output_file = self.output_dir / "stress_test_analysis.png"
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        print(f"   グラフ保存: {output_file}")
        plt.close()
    
    def generate_performance_score(self):
        """総合パフォーマンススコアを計算"""
        score = 100  # 満点から減点方式
        
        print("📊 総合パフォーマンススコア計算")
        
        # 起動時間評価 (目標: 3秒以下)
        startup_data = self.results['benchmarks']['startup_time']
        valid_times = [t for t in startup_data if t is not None]
        if valid_times:
            avg_startup = np.mean(valid_times)
            if avg_startup > 5:
                score -= 30
                print(f"   起動時間: 大幅減点 ({avg_startup:.2f}秒 > 5秒)")
            elif avg_startup > 3:
                score -= 15
                print(f"   起動時間: 減点 ({avg_startup:.2f}秒 > 3秒)")
            else:
                print(f"   起動時間: 良好 ({avg_startup:.2f}秒)")
        
        # メモリ使用量評価 (目標: 100MB以下)
        memory_data = self.results['benchmarks']['memory_usage']
        if memory_data:
            avg_memory = memory_data['avg_mb']
            if avg_memory > 200:
                score -= 25
                print(f"   メモリ使用量: 大幅減点 ({avg_memory:.1f}MB > 200MB)")
            elif avg_memory > 100:
                score -= 10
                print(f"   メモリ使用量: 減点 ({avg_memory:.1f}MB > 100MB)")
            else:
                print(f"   メモリ使用量: 良好 ({avg_memory:.1f}MB)")
        
        # API応答性能評価 (目標: 100ms以下)
        api_data = self.results['benchmarks']['api_response']
        if api_data:
            slow_apis = 0
            for item in api_data:
                if 'avg_response_time' in item and item['avg_response_time'] > 0.1:
                    slow_apis += 1
            if slow_apis > 0:
                score -= slow_apis * 5
                print(f"   API応答性能: 減点 ({slow_apis}個のAPIが100ms超過)")
            else:
                print("   API応答性能: 良好")
        
        # ストレステスト評価
        stress_data = self.results['benchmarks']['stress_test']
        if stress_data:
            if 'workspace_creation' in stress_data:
                ws_success_rate = stress_data['workspace_creation']['success_count'] / stress_data['workspace_creation']['target_count']
                if ws_success_rate < 0.8:
                    score -= 20
                    print(f"   ストレステスト: 減点 (ワークスペース作成成功率 {ws_success_rate*100:.1f}% < 80%)")
                else:
                    print(f"   ストレステスト: 良好 (ワークスペース作成成功率 {ws_success_rate*100:.1f}%)")
        
        score = max(0, score)  # 0点未満にはしない
        
        print(f"\n🎯 総合パフォーマンススコア: {score}/100")
        
        if score >= 90:
            print("   評価: 優秀 ⭐⭐⭐")
        elif score >= 70:
            print("   評価: 良好 ⭐⭐")
        elif score >= 50:
            print("   評価: 改善の余地あり ⭐")
        else:
            print("   評価: 要改善")
        
        return score
    
    def generate_comprehensive_report(self):
        """包括的なHTMLレポートを生成"""
        html_template = """
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WezTerm Parallel パフォーマンス分析レポート</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: #1e1e2e;
            color: #cdd6f4;
            margin: 0;
            padding: 20px;
            line-height: 1.6;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: #313244;
            border-radius: 12px;
            padding: 30px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }
        h1 {
            color: #89b4fa;
            text-align: center;
            font-size: 2.5em;
            margin-bottom: 30px;
        }
        h2 {
            color: #a6e3a1;
            border-bottom: 2px solid #a6e3a1;
            padding-bottom: 10px;
        }
        h3 {
            color: #f9e2af;
        }
        .metric-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }
        .metric-card {
            background: #45475a;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }
        .metric-value {
            font-size: 2em;
            font-weight: bold;
            color: #94e2d5;
        }
        .metric-label {
            color: #bac2de;
            margin-top: 5px;
        }
        .score-circle {
            width: 120px;
            height: 120px;
            border-radius: 50%;
            margin: 20px auto;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
            font-weight: bold;
            color: white;
        }
        .score-excellent { background: linear-gradient(45deg, #a6e3a1, #94e2d5); }
        .score-good { background: linear-gradient(45deg, #f9e2af, #fab387); }
        .score-average { background: linear-gradient(45deg, #fab387, #f38ba8); }
        .score-poor { background: linear-gradient(45deg, #f38ba8, #eba0ac); }
        .image-gallery {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }
        .image-card {
            background: #45475a;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
        }
        .image-card img {
            max-width: 100%;
            height: auto;
            border-radius: 4px;
        }
        .system-info {
            background: #45475a;
            padding: 20px;
            border-radius: 8px;
            margin: 20px 0;
        }
        .timestamp {
            text-align: center;
            color: #bac2de;
            font-style: italic;
            margin-bottom: 30px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>WezTerm Parallel パフォーマンス分析レポート</h1>
        <div class="timestamp">生成日時: {timestamp}</div>
        
        <h2>総合パフォーマンススコア</h2>
        <div class="score-circle {score_class}">
            {score}/100
        </div>
        
        <h2>システム情報</h2>
        <div class="system-info">
            {system_info}
        </div>
        
        <h2>主要メトリクス</h2>
        <div class="metric-grid">
            {metrics_cards}
        </div>
        
        <h2>詳細分析グラフ</h2>
        <div class="image-gallery">
            {analysis_images}
        </div>
        
        <h2>推奨改善点</h2>
        <div style="background: #45475a; padding: 20px; border-radius: 8px;">
            {recommendations}
        </div>
    </div>
</body>
</html>
        """
        
        # スコア計算
        score = self.generate_performance_score()
        
        # スコアに応じたクラス
        if score >= 90:
            score_class = "score-excellent"
        elif score >= 70:
            score_class = "score-good"
        elif score >= 50:
            score_class = "score-average"
        else:
            score_class = "score-poor"
        
        # システム情報
        system_info = "<ul>"
        for key, value in self.results['system_info'].items():
            system_info += f"<li><strong>{key}</strong>: {value}</li>"
        system_info += "</ul>"
        
        # メトリクスカード
        metrics_cards = ""
        
        # 起動時間カード
        startup_data = self.results['benchmarks']['startup_time']
        valid_times = [t for t in startup_data if t is not None]
        if valid_times:
            metrics_cards += f"""
            <div class="metric-card">
                <div class="metric-value">{np.mean(valid_times):.2f}s</div>
                <div class="metric-label">平均起動時間</div>
            </div>
            """
        
        # メモリ使用量カード
        memory_data = self.results['benchmarks']['memory_usage']
        if memory_data:
            metrics_cards += f"""
            <div class="metric-card">
                <div class="metric-value">{memory_data['avg_mb']:.1f}MB</div>
                <div class="metric-label">平均メモリ使用量</div>
            </div>
            """
        
        # API応答時間カード
        api_data = self.results['benchmarks']['api_response']
        if api_data:
            avg_response = np.mean([item['avg_response_time'] for item in api_data if 'avg_response_time' in item]) * 1000
            metrics_cards += f"""
            <div class="metric-card">
                <div class="metric-value">{avg_response:.1f}ms</div>
                <div class="metric-label">平均API応答時間</div>
            </div>
            """
        
        # 分析画像
        analysis_images = ""
        image_files = ['startup_time_analysis.png', 'memory_usage_analysis.png', 
                      'api_response_analysis.png', 'stress_test_analysis.png']
        
        for img_file in image_files:
            img_path = self.output_dir / img_file
            if img_path.exists():
                analysis_images += f"""
                <div class="image-card">
                    <h3>{img_file.replace('_', ' ').replace('.png', '').title()}</h3>
                    <img src="{img_file}" alt="{img_file}">
                </div>
                """
        
        # 推奨改善点
        recommendations = "<ul>"
        if valid_times and np.mean(valid_times) > 3:
            recommendations += "<li>起動時間の最適化: 遅延初期化の活用やプリロード処理の改善を検討してください</li>"
        if memory_data and memory_data['avg_mb'] > 100:
            recommendations += "<li>メモリ使用量の削減: メモリプールの最適化やガベージコレクションの調整を検討してください</li>"
        if api_data:
            slow_apis = [item for item in api_data if 'avg_response_time' in item and item['avg_response_time'] > 0.1]
            if slow_apis:
                recommendations += f"<li>API応答性能の改善: {len(slow_apis)}個のAPIが目標値(100ms)を超過しています</li>"
        if score < 70:
            recommendations += "<li>包括的な性能チューニング: 設定ファイルの見直しとパフォーマンス設定の最適化を推奨します</li>"
        
        if recommendations == "<ul>":
            recommendations += "<li>現在のパフォーマンスは良好です。定期的な監視を継続してください。</li>"
        recommendations += "</ul>"
        
        # HTMLファイル生成
        html_content = html_template.format(
            timestamp=datetime.now().strftime("%Y年%m月%d日 %H:%M:%S"),
            score=score,
            score_class=score_class,
            system_info=system_info,
            metrics_cards=metrics_cards,
            analysis_images=analysis_images,
            recommendations=recommendations
        )
        
        html_file = self.output_dir / "performance_report.html"
        with open(html_file, 'w', encoding='utf-8') as f:
            f.write(html_content)
        
        print(f"📄 包括的レポート生成: {html_file}")
    
    def run_complete_analysis(self):
        """完全な分析を実行"""
        print("🚀 WezTerm Parallel パフォーマンス分析開始")
        print(f"📁 結果ファイル: {self.results_file}")
        print(f"📁 出力ディレクトリ: {self.output_dir}")
        print()
        
        self.analyze_startup_time()
        print()
        self.analyze_memory_usage()
        print()
        self.analyze_api_response()
        print()
        self.analyze_stress_test()
        print()
        self.generate_comprehensive_report()
        
        print("\n✅ 分析完了!")
        print(f"📊 分析結果: {self.output_dir}/")
        print(f"📄 HTMLレポート: {self.output_dir}/performance_report.html")

def main():
    parser = argparse.ArgumentParser(description='WezTerm Parallel パフォーマンス分析ツール')
    parser.add_argument('results_file', help='ベンチマーク結果JSONファイル')
    parser.add_argument('--startup-only', action='store_true', help='起動時間分析のみ実行')
    parser.add_argument('--memory-only', action='store_true', help='メモリ分析のみ実行')
    parser.add_argument('--api-only', action='store_true', help='API分析のみ実行')
    parser.add_argument('--score-only', action='store_true', help='スコア計算のみ実行')
    
    args = parser.parse_args()
    
    try:
        analyzer = PerformanceAnalyzer(args.results_file)
        
        if args.startup_only:
            analyzer.analyze_startup_time()
        elif args.memory_only:
            analyzer.analyze_memory_usage()
        elif args.api_only:
            analyzer.analyze_api_response()
        elif args.score_only:
            analyzer.generate_performance_score()
        else:
            analyzer.run_complete_analysis()
            
    except KeyboardInterrupt:
        print("\n❌ 分析が中断されました")
        sys.exit(1)
    except Exception as e:
        print(f"❌ 分析中にエラーが発生: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()