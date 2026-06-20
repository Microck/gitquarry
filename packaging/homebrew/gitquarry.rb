class Gitquarry < Formula
  desc "Terminal CLI for public GitHub repository search with explicit discovery controls"
  homepage "https://github.com/Microck/gitquarry"
  version "0.1.10"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.10/gitquarry-v0.1.10-aarch64-apple-darwin.tar.gz"
      sha256 "f4cc1b3d54b5c576fc306085ebae89c53c0bdad29a348d9ed38a6af2dafb12d6"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.10/gitquarry-v0.1.10-x86_64-apple-darwin.tar.gz"
      sha256 "8e26541354be175d6970238113f2d7d796a708190c723bd43e983eaa29baf0cb"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.10/gitquarry-v0.1.10-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "334b0fa8c6df47d6904ba4dc4887b1f4da456a01323e44f39a78c2016ecdcfdb"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.10/gitquarry-v0.1.10-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "34c4b06ffccab1adfa2270b84a9ba5175e178c162d213715f6a0f911db426403"
    end
  end

  def install
    bin.install "gitquarry"
  end

  test do
    assert_match "Usage: gitquarry [OPTIONS] [COMMAND]", shell_output("#{bin}/gitquarry --help")
  end
end
