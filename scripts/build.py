import argparse
import os
import sh
import shutil
import toml
import zipfile

APP_NAME = "meters"
MACOS_APP_NAME = "MetersBelowTheGround"
README_NAME = "README.md"
LICENSE_NAME = "LICENSE"

BINARY_NAMES = { "unix": "%s_unix" % APP_NAME,
                 "glutin": "%s_glutin" % APP_NAME,
               }

OUTPUT_NAMES = { "unix": "%s-terminal" % APP_NAME,
                 "glutin": "%s-opengl" % APP_NAME,
               }

SH_KWARGS = { "_out": "/dev/stdout",
              "_err": "/dev/stderr",
              "_fg": True,
            }

def make_parser():
  parser = argparse.ArgumentParser()
  parser.add_argument('--frontend', action='append')
  parser.add_argument('--build-path')
  parser.add_argument('--upload-path')
  parser.add_argument('--crate-path', action='append')
  parser.add_argument('--root-path')
  parser.add_argument('--os')
  return parser

def build_common(args):
  args.architecture = "x86_64"

  for manifest_path in args.manifest_path:
    sh.cargo.build(
      "--manifest-path", manifest_path,
      "--release",
      **SH_KWARGS)

  output_dir_name = "%(app)s-%(os)s-%(architecture)s-v%(version)s" % {
      "app": APP_NAME,
      "os": args.os,
      "architecture": args.architecture,
      "version": args.version,
  }
  output_dir_path = os.path.join(args.build_path, output_dir_name)
  os.makedirs(output_dir_path)

  for frontend in args.frontend:
    shutil.copy(
        os.path.join("target", "release", BINARY_NAMES[frontend]),
        os.path.join(output_dir_path, OUTPUT_NAMES[frontend]))

  shutil.copy(
      os.path.join(args.root_path, README_NAME),
      os.path.join(output_dir_path, "README.txt"))
  shutil.copy(
      os.path.join(args.root_path, LICENSE_NAME),
      os.path.join(output_dir_path, "LICENSE.txt"))
  sh.git("rev-parse", "HEAD", _err="/dev/stderr",
      _out=os.path.join(output_dir_path, "REVISION.txt"))

  if not os.path.exists(args.upload_path):
    os.makedirs(args.upload_path)

  zip_name = "%s.zip" % output_dir_name
  zip_path = os.path.join(args.upload_path, zip_name)

  with zipfile.ZipFile(zip_path, 'w') as zip_file:
    for subdir, dirs, files in os.walk(output_dir_path):
      for f in files:
        arcname = os.path.join(os.path.basename(subdir), f)
        zip_file.write(os.path.join(subdir, f), arcname)

  revision_zip_name = "%(app)s-%(os)s-%(architecture)s-%(branch)s.zip" % {
      "app": APP_NAME,
      "os": args.os,
      "architecture": args.architecture,
      "branch": args.branch,
  }

  shutil.copy(zip_path, os.path.join(args.upload_path, revision_zip_name))

  if args.os == "macos" and "glutin" in args.frontend:
    args.output_dir_path = output_dir_path
    args.bin_path = os.path.join(output_dir_path, OUTPUT_NAMES["glutin"])
    make_macos_app(args)

def make_macos_app(args):
  app_dir_name = "%s.app" % MACOS_APP_NAME
  dmg_dir_path = os.path.join(args.build_path, MACOS_APP_NAME)
  app_dir_path = os.path.join(dmg_dir_path, app_dir_name)
  full_dir_path = os.path.join(app_dir_path, "Contents", "MacOS")

  os.makedirs(full_dir_path)

  shutil.copy(args.bin_path, os.path.join(full_dir_path, MACOS_APP_NAME))
  shutil.copy(
      os.path.join(args.output_dir_path, "README.txt"),
      os.path.join(dmg_dir_path, "README.txt"))
  shutil.copy(
      os.path.join(args.output_dir_path, "LICENSE.txt"),
      os.path.join(dmg_dir_path, "LICENSE.txt"))
  sh.git("rev-parse", "HEAD", _err="/dev/stderr",
      _out=os.path.join(dmg_dir_path, "REVISION.txt"))
  os.symlink("/Applications", os.path.join(dmg_dir_path, "Applications"))

  dmg_name = "%s-v%s.dmg" % (MACOS_APP_NAME, args.version)

  sh.hdiutil.create(
      os.path.join(args.upload_path, dmg_name),
      "-srcfolder", dmg_dir_path)

  revision_dmg_name = "%s-%s.dmg" % (MACOS_APP_NAME, args.branch)
  shutil.copy(
      os.path.join(args.upload_path, dmg_name),
      os.path.join(args.upload_path, revision_dmg_name))

def build_wasm(args):
  crate_paths = [os.path.normpath(crate_path) for crate_path in args.crate_path]
  for crate_path in crate_paths:
    sh.npm("install", "--prefix", crate_path, **SH_KWARGS)
    sh.bash(os.path.join(crate_path, "build_dist.sh"), **SH_KWARGS)

    output_dir_path = os.path.join(args.upload_path, APP_NAME)
    os.makedirs(output_dir_path)
    shutil.copytree(
        os.path.join(crate_path, "dist"),
        os.path.join(output_dir_path, "v%s" % args.version))

    shutil.copytree(
        os.path.join(crate_path, "dist"),
        os.path.join(output_dir_path, "%s" % args.branch))

def main(args):
  args.crate_path = [os.path.normpath(crate_path) for crate_path in args.crate_path]
  args.build_path = os.path.normpath(args.build_path)
  args.upload_path = os.path.normpath(args.upload_path)
  args.root_path = os.path.normpath(args.root_path)
  args.manifest_path = [os.path.join(crate_path, "Cargo.toml") for crate_path in args.crate_path]
  args.manifest = toml.load(args.manifest_path)
  args.version = args.manifest["package"]["version"]

  try:
    args.branch = os.environ["TRAVIS_BRANCH"]
  except KeyError:
    args.branch = sh.git("rev-parse", "--abbrev-ref", "HEAD").strip()

  if "unix" in args.frontend or "glutin" in args.frontend:
    build_common(args)

  if "wasm" in args.frontend:
    build_wasm(args)

if __name__ == "__main__":
  parser = make_parser()
  main(parser.parse_args())
