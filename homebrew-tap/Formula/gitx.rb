# frozen_string_literal: true

# Documentation: https://docs.brew.sh/Formula-Cookbook
class Gitx < Formula
  desc "AI-Powered Git Diff Analyzer for macOS and Linux"
  homepage "https://github.com/itbaby/gitx"
  url "https://github.com/itbaby/gitx/releases/download/v0.1.1/gitx-0.1.1.tar.gz"
  version "0.1.1"
  license "MIT"
  head "https://github.com/itbaby/gitx.git", branch: "main"

  on_macos do
    on_intel do
      url "https://github.com/itbaby/gitx/releases/download/v0.1.1/GitX_0.1.1_x86_64.dmg"
      sha256 "X86_64_SHA256_PLACEHOLDER"
    end

    on_arm do
      url "https://github.com/itbaby/gitx/releases/download/v0.1.1/GitX_0.1.1_aarch64.dmg"
      sha256 "ARM64_SHA256_PLACEHOLDER"
    end
  end

  on_linux do
    url "https://github.com/itbaby/gitx/releases/download/v0.1.1/gitx_0.1.1_amd64.deb"
    sha256 "LINUX_SHA256_PLACEHOLDER"
  end

  def install
    if OS.mac?
      # For DMG, we need to mount and copy the .app bundle
      require "open3"
      
      # Create temporary mount point
      mount_point = Dir.mktmpdir("gitx-dmg")
      
      # Mount the DMG
      _, status = Open3.capture2e("hdiutil", "attach", downloaded_cache_path.to_s, "-mountpoint", mount_point, "-nobrowse")
      
      if status.success?
        # Find and copy the .app bundle
        app_path = File.join(mount_point, "GitX.app")
        if File.exist?(app_path)
          FileUtils.cp_r(app_path, "/Applications/")
        else
          # Try finding in subdirectories
          found_app = Dir.glob(File.join(mount_point, "**", "GitX.app")).first
          FileUtils.cp_r(found_app, "/Applications/") if found_app
        end
        
        # Unmount the DMG
        system "hdiutil", "detach", mount_point
      end
      
      Dir.rmdir(mount_point) if Dir.exist?(mount_point)
    else
      # Linux: install deb package
      system "dpkg", "-i", "-force-all", downloaded_cache_path.to_s
    end
  end

  def post_install
    return unless OS.mac?

    # Remove quarantine attribute to allow app to run
    app_path = Pathname.new("/Applications/GitX.app")
    system "xattr", "-dr", "com.apple.quarantine", app_path.to_s if app_path.exist?
  end

  test do
    # Test that the app bundle exists
    assert_predicate "/Applications/GitX.app", :exist?
  end
end
