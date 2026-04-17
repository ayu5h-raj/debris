# Homebrew formula for sweep
# To publish: copy this to ayu5h-raj/homebrew-tap/Formula/sweep.rb
# and replace the SHA256 placeholders with actual values from the release artifacts.
class Sweep < Formula
  desc "Minimal Mac storage cleaner — find and delete orphaned app data"
  homepage "https://github.com/ayu5h-raj/sweep"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/ayu5h-raj/sweep/releases/download/v0.1.0/sweep-macos-arm64-v0.1.0.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    end
    on_intel do
      url "https://github.com/ayu5h-raj/sweep/releases/download/v0.1.0/sweep-macos-intel-v0.1.0.tar.gz"
      sha256 "REPLACE_WITH_INTEL_SHA256"
    end
  end

  def install
    bin.install "sweep"
  end

  test do
    assert_match "sweep", shell_output("#{bin}/sweep --version 2>&1", 1)
  end
end
