class Gitquarry < Formula
  desc "Terminal CLI for public GitHub repository search with explicit discovery controls"
  homepage "https://github.com/Microck/gitquarry"
  version "0.1.8"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.8/gitquarry-v0.1.8-aarch64-apple-darwin.tar.gz"
      sha256 "7fa7b07e12eead5bcff4206c6e8506bc7920243990d6c82df20313c2c639f799"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.8/gitquarry-v0.1.8-x86_64-apple-darwin.tar.gz"
      sha256 "675a2236123ec32a778aba95c471eb0832f9e099a66ff4dee596f3b6aabd42a6"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.8/gitquarry-v0.1.8-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "aa17d2ce5f7861a38108528efab2d0c8ee2d642dc6b2b9205238fdf7f15e532d"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.8/gitquarry-v0.1.8-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "d8142f8dad82f1263c188baa1a4283bb1dd1fde8c617f8031e1b581edf8ae421"
    end
  end

  def install
    bin.install "gitquarry"
  end

  test do
    assert_match "Usage: gitquarry [OPTIONS] [COMMAND]", shell_output("#{bin}/gitquarry --help")
  end
end
