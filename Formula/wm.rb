class Wm < Formula
  desc "Working memory for AI coding assistants"
  homepage "https://github.com/cloud-atlas-ai/wm"
  url "https://github.com/cloud-atlas-ai/wm/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "502b217da3e844341bcbde081f7af0f0345fc7b485f089b92d23c7fdc4e5ed3d"
  license :cannot_represent

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "wm", shell_output("#{bin}/wm --help")
  end
end
