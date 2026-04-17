class Debris < Formula
  desc "Minimal Mac storage cleaner — find and delete orphaned app data"
  homepage "https://github.com/ayu5h-raj/debris"
  version "0.3.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/ayu5h-raj/debris/releases/download/v0.3.0/debris-macos-arm64-v0.3.0.tar.gz"
      sha256 "3bad7bbece0ca93d31cb5c6158838c84308eaff87311493abe9b703cd5494eae"
    end
    on_intel do
      url "https://github.com/ayu5h-raj/debris/releases/download/v0.3.0/debris-macos-intel-v0.3.0.tar.gz"
      sha256 "2b159821d7815192d8fc0ea93766d9e02ed6db643705e16de17838c66ade1070"
    end
  end

  def install
    bin.install "debris"
  end

  test do
    assert_predicate bin/"debris", :executable
  end
end
