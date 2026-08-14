class Gitquarry < Formula
  desc "Terminal CLI for public GitHub repository search with explicit discovery controls"
  homepage "https://github.com/Microck/gitquarry"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.2.0/gitquarry-v0.2.0-aarch64-apple-darwin.tar.gz"
      sha256 "dc403e89ee298a4a61de4d7cf2eef1be3f3cde6caf48e4ed917e44150cd6a00b"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.2.0/gitquarry-v0.2.0-x86_64-apple-darwin.tar.gz"
      sha256 "5913f6de7cd324c324f0f6ef2f7a84c7d99462167e3d1a5c9fae9faaecc5bcf8"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.2.0/gitquarry-v0.2.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "aef1d3aef0dc0858c99c1c69c983f2e53f69c3b684ac4abb78d1bc63d8401678"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.2.0/gitquarry-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "9abdadeebe51e06ac958927d5a0547a215c929858276f196a7bdba2e770d55fd"
    end
  end

  def install
    bin.install "gitquarry"
  end

  test do
    assert_match "Usage: gitquarry [OPTIONS] [COMMAND]", shell_output("#{bin}/gitquarry --help")
  end
end
