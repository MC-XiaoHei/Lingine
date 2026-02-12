import os
import subprocess

SOURCE_DIR = r"./run/datasets/raw"
OUTPUT_DIR = r"./run/datasets"
GDAL_CMD = "gdalwarp"

def preprocess():
    print(f"Starting Smart Preprocessing...")
    print(f"Source: {os.path.abspath(SOURCE_DIR)}")
    print(f"Target: {os.path.abspath(OUTPUT_DIR)}\n")

    count = 0

    for root, dirs, files in os.walk(SOURCE_DIR):
        for filename in files:
            if not filename.lower().endswith(('.tif', '.tiff')):
                continue

            count += 1

            src_path = os.path.join(root, filename)
            rel_path = os.path.relpath(src_path, SOURCE_DIR)

            out_path = os.path.join(OUTPUT_DIR, rel_path)

            out_folder = os.path.dirname(out_path)
            if not os.path.exists(out_folder):
                os.makedirs(out_folder, exist_ok=True)

            name_lower = filename.lower()

            resample_alg = "bilinear"
            type_tag = "Continuous"

            if any(x in name_lower for x in ["esa", "worldcover", "class", "id", "type", "mask"]):
                resample_alg = "near"
                type_tag = "Categorical"

            print(f"[{count}] {rel_path}")
            print(f"    ├─ Type: {type_tag} -> Algo: {resample_alg}")

            cmd = [
                GDAL_CMD,
                "-t_srs", "EPSG:4326",
                "-r", resample_alg,
                "-multi", "-wo", "NUM_THREADS=ALL_CPUS",
                "-co", "COMPRESS=ZSTD",
                "-co", "PREDICTOR=2",
                "-co", "TILED=YES",
                "-co", "BLOCKXSIZE=256",
                "-co", "BLOCKYSIZE=256",
                "-co", "BIGTIFF=IF_NEEDED",
                "-overwrite",
                "-q",
                src_path,
                out_path
            ]

            try:
                subprocess.run(cmd, check=True)
            except subprocess.CalledProcessError:
                print(f"    FAILED: {filename}")
            except FileNotFoundError:
                print("    ERROR: gdalwarp command not found!")
                return

    print(f"\nDone! Processed {count} files.")

if __name__ == "__main__":
    preprocess()