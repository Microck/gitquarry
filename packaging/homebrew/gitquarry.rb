class Gitquarry < Formula
  desc "Terminal CLI for public GitHub repository search with explicit discovery controls"
  homepage "https://github.com/Microck/gitquarry"
  version "0.1.6"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.6/gitquarry-v0.1.6-aarch64-apple-darwin.tar.gz"
      sha256 "2a0b4814cc862402e32f3b0a4f39718f337079595de94f9de568c82deb3b492d"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.6/gitquarry-v0.1.6-x86_64-apple-darwin.tar.gz"
      sha256 "e5a4301519c2a1f87c8f3c90dfdf958876e6386a7d4c7f59a882c27426f3ca07"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.6/gitquarry-v0.1.6-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "d25d2a1ffc44b170505c4219309388eccc3d55d43c6739d498bc7df8e6325b4b"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.6/gitquarry-v0.1.6-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "18f30b40ab5c48b2fe3b8273b735b7aed0333c886a467384f595e170e772a7ee"
    end
  end

  def install
    bin.install "gitquarry"
  end

  test do
    assert_match "Usage: gitquarry [OPTIONS] [COMMAND]", shell_output("#{bin}/gitquarry --help")
  end
end
