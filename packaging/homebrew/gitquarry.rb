class Gitquarry < Formula
  desc "Terminal CLI for public GitHub repository search with explicit discovery controls"
  homepage "https://github.com/Microck/gitquarry"
  version "0.1.9"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.9/gitquarry-v0.1.9-aarch64-apple-darwin.tar.gz"
      sha256 "e4e1df812a6472331e2a34ff6fbd83a7cf1918a97b3e251cead9f748d6b6bf60"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.9/gitquarry-v0.1.9-x86_64-apple-darwin.tar.gz"
      sha256 "43170f2a79d273c0bb847a09b38d5a6a7655e818d1e04e3ce15f384b0dc69c8c"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.9/gitquarry-v0.1.9-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "d17b90152b2a6419fc4fb07fde76462653fd37fe72f2bd89f776b75e84110a60"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.9/gitquarry-v0.1.9-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "db8aa0afcbf620e8d18d1e83ca881a7c8ca1f8702f6be255a8d64cf9d2569f63"
    end
  end

  def install
    bin.install "gitquarry"
  end

  test do
    assert_match "Usage: gitquarry [OPTIONS] [COMMAND]", shell_output("#{bin}/gitquarry --help")
  end
end
