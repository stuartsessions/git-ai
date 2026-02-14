#!/usr/bin/env python3
"""
Benchmark cargo build and test performance across branches, platforms, and linkers.
Compares main vs staging branches to evaluate refactoring impact.
"""

import subprocess
import platform
import time
import json
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import shutil


class BenchmarkRunner:
    def __init__(self, repo_root: Path, output_dir: Path):
        self.repo_root = repo_root
        self.output_dir = output_dir
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.platform_name = self._detect_platform()
        self.time_cmd = self._detect_time_command()
        self.results = []
        
    def _detect_platform(self) -> str:
        """Detect the current platform."""
        system = platform.system()
        if system == "Linux":
            return "Linux"
        elif system == "Darwin":
            return "macOS"
        elif system == "Windows":
            return "Windows"
        else:
            return system
    
    def _detect_time_command(self) -> Optional[List[str]]:
        """Detect which time utility to use for detailed stats."""
        if self.platform_name == "Windows":
            return None  # Windows doesn't have GNU time
        
        # Check for gtime (GNU time on macOS via Homebrew)
        if shutil.which("gtime"):
            return ["gtime", "-v"]
        
        # Check for /usr/bin/time (GNU time on Linux)
        if os.path.exists("/usr/bin/time"):
            return ["/usr/bin/time", "-v"]
        
        return None
    
    def _get_linkers_for_platform(self) -> List[Tuple[str, str]]:
        """
        Get available linkers for the current platform.
        Returns list of (name, rustflags) tuples.
        """
        if self.platform_name == "Linux":
            return [
                ("default", ""),
                ("lld", "-C link-arg=-fuse-ld=lld"),
                ("mold", "-C link-arg=-fuse-ld=mold"),
            ]
        elif self.platform_name == "macOS":
            return [
                ("default", ""),
                ("lld", "-C link-arg=-fuse-ld=lld"),
            ]
        elif self.platform_name == "Windows":
            return [
                ("default", ""),
                ("lld", "-C link-arg=-fuse-ld=lld"),
            ]
        else:
            return [("default", "")]
    
    def _run_with_time(self, cmd: List[str], env: Dict[str, str]) -> Tuple[float, int, str]:
        """
        Run a command and measure time and memory usage.
        Returns (elapsed_seconds, peak_memory_mb, stdout).
        """
        if self.time_cmd and self.platform_name != "Windows":
            # Use GNU time for detailed stats
            full_cmd = self.time_cmd + cmd
            try:
                result = subprocess.run(
                    full_cmd,
                    cwd=self.repo_root,
                    env=env,
                    capture_output=True,
                    text=True
                )
                
                # Parse output from GNU time (in stderr)
                stderr = result.stderr
                
                # Extract elapsed time
                elapsed_seconds = 0.0
                for line in stderr.split('\n'):
                    if "Elapsed (wall clock) time" in line:
                        time_str = line.split(":")[-1].strip()
                        # Parse format like "0:05.23" or "1:23.45"
                        parts = time_str.split(':')
                        if len(parts) == 2:
                            mins, secs = parts
                            elapsed_seconds = int(mins) * 60 + float(secs)
                
                # Extract peak memory (in KB on Linux/Mac)
                peak_kb = 0
                for line in stderr.split('\n'):
                    if "Maximum resident set size" in line:
                        peak_kb = int(line.split(":")[-1].strip())
                        break
                
                peak_mb = peak_kb // 1024
                
                return elapsed_seconds, peak_mb, result.stdout
            except Exception as e:
                print(f"Warning: GNU time failed, falling back to basic timing: {e}")
        
        # Fallback: basic timing without memory stats
        start = time.time()
        result = subprocess.run(
            cmd,
            cwd=self.repo_root,
            env=env,
            capture_output=True,
            text=True
        )
        elapsed = time.time() - start
        
        return elapsed, 0, result.stdout
    
    def _run_cargo_clean(self):
        """Clean cargo build artifacts."""
        subprocess.run(
            ["cargo", "clean"],
            cwd=self.repo_root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
    
    def _get_current_branch(self) -> str:
        """Get the current git branch name."""
        result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            cwd=self.repo_root,
            capture_output=True,
            text=True
        )
        return result.stdout.strip()
    
    def _checkout_branch(self, branch: str):
        """Checkout a git branch."""
        print(f"  Checking out branch: {branch}")
        result = subprocess.run(
            ["git", "checkout", branch],
            cwd=self.repo_root,
            capture_output=True,
            text=True
        )
        if result.returncode != 0:
            raise RuntimeError(f"Failed to checkout {branch}: {result.stderr}")
    
    def benchmark_configuration(
        self,
        branch: str,
        linker_name: str,
        rustflags: str
    ) -> Dict:
        """
        Run benchmark for a specific configuration.
        Returns result dictionary.
        """
        print(f"\n  Benchmarking: branch={branch}, linker={linker_name}")
        
        # Prepare environment
        env = os.environ.copy()
        if rustflags:
            env["RUSTFLAGS"] = rustflags
        
        # Clean before build
        self._run_cargo_clean()
        
        # Benchmark cargo build
        print(f"    Running cargo build...")
        build_time, build_mem, _ = self._run_with_time(
            ["cargo", "build", "--release"],
            env
        )
        
        # Benchmark cargo test
        print(f"    Running cargo test...")
        test_time, test_mem, _ = self._run_with_time(
            ["cargo", "test", "--release"],
            env
        )
        
        result = {
            "timestamp": datetime.now().isoformat(),
            "platform": self.platform_name,
            "branch": branch,
            "linker": linker_name,
            "rustflags": rustflags,
            "build_time_seconds": round(build_time, 2),
            "test_time_seconds": round(test_time, 2),
            "total_time_seconds": round(build_time + test_time, 2),
            "build_memory_mb": build_mem,
            "test_memory_mb": test_mem,
        }
        
        print(f"    Build: {build_time:.2f}s, Test: {test_time:.2f}s")
        
        return result
    
    def run_all_benchmarks(self, branches: List[str]) -> List[Dict]:
        """
        Run benchmarks for all combinations of branches and linkers.
        """
        print(f"Starting benchmarks on {self.platform_name}")
        print(f"Branches: {', '.join(branches)}")
        
        if not self.time_cmd:
            print("Warning: GNU time not found. Memory stats will not be available.")
            if self.platform_name == "macOS":
                print("  Install with: brew install gnu-time")
            elif self.platform_name == "Linux":
                print("  Install with: sudo apt install time")
        
        # Save original branch to restore later
        original_branch = self._get_current_branch()
        print(f"Original branch: {original_branch}")
        
        linkers = self._get_linkers_for_platform()
        print(f"Linkers to test: {', '.join(name for name, _ in linkers)}")
        
        results = []
        
        try:
            for branch in branches:
                self._checkout_branch(branch)
                
                for linker_name, rustflags in linkers:
                    try:
                        result = self.benchmark_configuration(
                            branch, linker_name, rustflags
                        )
                        results.append(result)
                    except Exception as e:
                        print(f"Error benchmarking {branch}/{linker_name}: {e}")
                        continue
        
        finally:
            # Restore original branch
            print(f"\nRestoring original branch: {original_branch}")
            self._checkout_branch(original_branch)
        
        self.results = results
        return results
    
    def save_results(self):
        """Save results to JSON and generate reports."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Save raw JSON
        json_file = self.output_dir / f"benchmark_{self.platform_name}_{timestamp}.json"
        with open(json_file, 'w') as f:
            json.dump(self.results, f, indent=2)
        print(f"\nResults saved to: {json_file}")
        
        # Generate and save human-readable report
        report_file = self.output_dir / f"benchmark_{self.platform_name}_{timestamp}.txt"
        report = self._generate_report()
        with open(report_file, 'w') as f:
            f.write(report)
        print(f"Report saved to: {report_file}")
        
        # Print report to console
        print("\n" + "="*80)
        print(report)
        print("="*80)
    
    def _generate_report(self) -> str:
        """Generate a human-readable comparison report."""
        lines = []
        lines.append(f"Benchmark Results - {self.platform_name}")
        lines.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        lines.append("="*80)
        lines.append("")
        
        if not self.results:
            lines.append("No results to display.")
            return "\n".join(lines)
        
        # Group results by linker for easier comparison
        linkers = sorted(set(r["linker"] for r in self.results))
        branches = sorted(set(r["branch"] for r in self.results))
        
        for linker in linkers:
            lines.append(f"\nLinker: {linker.upper()}")
            lines.append("-"*80)
            
            # Table header
            lines.append(f"{'Branch':<15} {'Build Time':<15} {'Test Time':<15} {'Total Time':<15} {'Build Mem':<12}")
            lines.append("-"*80)
            
            linker_results = [r for r in self.results if r["linker"] == linker]
            
            for result in linker_results:
                branch = result["branch"]
                build_time = f"{result['build_time_seconds']:.2f}s"
                test_time = f"{result['test_time_seconds']:.2f}s"
                total_time = f"{result['total_time_seconds']:.2f}s"
                build_mem = f"{result['build_memory_mb']}MB" if result['build_memory_mb'] else "N/A"
                
                lines.append(f"{branch:<15} {build_time:<15} {test_time:<15} {total_time:<15} {build_mem:<12}")
            
            # Add comparison if we have both branches
            if len(branches) == 2:
                main_result = next((r for r in linker_results if r["branch"] == "main"), None)
                staging_result = next((r for r in linker_results if r["branch"] == "staging"), None)
                
                if main_result and staging_result:
                    lines.append("")
                    lines.append("Comparison (staging vs main):")
                    
                    build_diff = staging_result["build_time_seconds"] - main_result["build_time_seconds"]
                    build_pct = (build_diff / main_result["build_time_seconds"]) * 100 if main_result["build_time_seconds"] > 0 else 0
                    
                    test_diff = staging_result["test_time_seconds"] - main_result["test_time_seconds"]
                    test_pct = (test_diff / main_result["test_time_seconds"]) * 100 if main_result["test_time_seconds"] > 0 else 0
                    
                    total_diff = staging_result["total_time_seconds"] - main_result["total_time_seconds"]
                    total_pct = (total_diff / main_result["total_time_seconds"]) * 100 if main_result["total_time_seconds"] > 0 else 0
                    
                    def format_diff(diff, pct):
                        sign = "+" if diff > 0 else ""
                        emoji = "📈" if diff > 0 else "📉" if diff < 0 else "➡️"
                        return f"{sign}{diff:.2f}s ({sign}{pct:.1f}%) {emoji}"
                    
                    lines.append(f"  Build time:  {format_diff(build_diff, build_pct)}")
                    lines.append(f"  Test time:   {format_diff(test_diff, test_pct)}")
                    lines.append(f"  Total time:  {format_diff(total_diff, total_pct)}")
        
        return "\n".join(lines)


def main():
    # Determine repository root
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent.parent  # Go up to project root
    
    # Output directory
    output_dir = repo_root / "benchmark-results"
    
    # Branches to compare
    branches = ["main", "staging"]
    
    print(f"Repository: {repo_root}")
    print(f"Output directory: {output_dir}")
    
    # Run benchmarks
    runner = BenchmarkRunner(repo_root, output_dir)
    runner.run_all_benchmarks(branches)
    runner.save_results()
    
    print("\n✅ Benchmarking complete!")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n⚠️  Benchmark interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        sys.exit(1)
