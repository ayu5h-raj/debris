class Debris < Formula
  desc "Minimal Mac storage cleaner — find and delete orphaned app data"
  homepage "https://github.com/ayu5h-raj/debris"
  version "0.2.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/ayu5h-raj/debris/releases/download/v0.2.0/debris-macos-arm64-v0.2.0.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    end
    on_intel do
      url "https://github.com/ayu5h-raj/debris/releases/download/v0.2.0/debris-macos-intel-v0.2.0.tar.gz"
      sha256 "REPLACE_WITH_INTEL_SHA256"
    end
  end

  def install
    bin.install "debris"
  end

  test do
    assert_predicate bin/"debris", :executable
  end
end
