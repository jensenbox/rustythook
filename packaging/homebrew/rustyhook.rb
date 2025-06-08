class Rustyhook < Formula
  desc "Blazing-fast, Rust-native Git hook runner"
  homepage "https://github.com/your-org/rustyhook"
  url "https://github.com/your-org/rustyhook/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  license "MIT"
  head "https://github.com/your-org/rustyhook.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
    # Install shell completions
    generate_completions_from_executable(bin/"rh", "completions")
  end

  test do
    system "#{bin}/rh", "--version"
  end
end