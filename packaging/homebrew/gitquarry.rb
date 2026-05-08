class Gitquarry < Formula
  desc "Terminal CLI for public GitHub repository search with explicit discovery controls"
  homepage "https://github.com/Microck/gitquarry"
  version "0.1.7"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.7/gitquarry-v0.1.7-aarch64-apple-darwin.tar.gz"
      sha256 "dd6f8531dd6eb97a07cbadf5d9c3dafe771ec799441e1f921046811d752b443a"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.7/gitquarry-v0.1.7-x86_64-apple-darwin.tar.gz"
      sha256 "5fbd5a219701baac0f821f1ae1642b59afeec6c8631409fd149363bd8e716e9c"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.7/gitquarry-v0.1.7-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0a20bac3bf6b3fc6c3119f5c570f11b5f4413bbc05646addff370f4fe0edb33d"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.7/gitquarry-v0.1.7-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "ee0b7ed7e3c9367edbb8728168f613b4b3759573001e49eb35f23fbf46240197"
    end
  end

  def install
    bin.install "gitquarry"
  end

  test do
    assert_match "Usage: gitquarry [OPTIONS] [COMMAND]", shell_output("#{bin}/gitquarry --help")
  end
end
