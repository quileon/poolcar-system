import matplotlib.patches as mpatches
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from sklearn.metrics.pairwise import haversine_distances

COLUMNS_TO_KEEP = [
    "test_id",
    "location_latitude",
    "location_longitude",
    "datetime_send",
    "datetime_receive",
    "satellites_visible",
    "satellites_used",
    "satellites_carrier_to_noise",
    "dop_hdop",
    "dop_pdop",
    "dop_vdop",
    "payload",
]
EARTH_RADIUS_M = 6_371_000


def pengujian_akurasi_gps(
    start_test_id,
    end_test_id,
    reference_gps_start_test_id,
    reference_gps_end_test_id,
    csv_file,
):
    df = pd.read_csv(csv_file, sep=";")
    reference_gps_set = df[
        (df["test_id"] >= reference_gps_start_test_id)
        & (df["test_id"] <= reference_gps_end_test_id)
    ]

    reference_latitude = reference_gps_set["location_latitude"].mean()
    reference_longitude = reference_gps_set["location_longitude"].mean()
    reference_coordinates = np.radians([[reference_latitude, reference_longitude]])

    test_set = df[
        (df["test_id"] >= start_test_id) & (df["test_id"] <= end_test_id)
    ].copy()
    test_set["datetime_send"] = pd.to_datetime(test_set["datetime_iso8601"], utc=True)
    test_set["datetime_receive"] = pd.to_datetime(test_set["received_at"], utc=True)
    columns = [c for c in COLUMNS_TO_KEEP if c in test_set.columns]
    test_set = test_set[columns]

    # Fill missing values in quality metrics with column mean
    for col in ["dop_hdop", "satellites_carrier_to_noise"]:
        if col in test_set.columns:
            n_missing = test_set[col].isna().sum()
            if n_missing > 0:
                col_mean = test_set[col].mean()
                test_set[col] = test_set[col].fillna(col_mean)
                print(
                    f"Filled {n_missing} missing value(s) in '{col}' with mean ({col_mean:.4f})"
                )

    test_coordinates = np.radians(
        test_set[["location_latitude", "location_longitude"]].values
    )

    error_distances = (
        haversine_distances(reference_coordinates, test_coordinates).flatten()
        * EARTH_RADIUS_M
    )
    test_set["error_distance_m"] = error_distances

    # Variable
    cep_50 = test_set["error_distance_m"].quantile(0.5)
    average_error = test_set["error_distance_m"].mean()
    min_error = test_set["error_distance_m"].min()
    max_error = test_set["error_distance_m"].max()
    average_cn0 = test_set["satellites_carrier_to_noise"].mean()
    min_cn0 = test_set["satellites_carrier_to_noise"].min()
    max_cn0 = test_set["satellites_carrier_to_noise"].max()
    average_hdop = test_set["dop_hdop"].mean()
    min_hdop = test_set["dop_hdop"].min()
    max_hdop = test_set["dop_hdop"].max()
    total_test = test_set.shape[0]

    # Print Summary
    print(f"Reference GPS : {reference_latitude}, {reference_longitude}")
    print(f"Total Test    : {total_test} tests")

    ## Error Summary
    print(f"CEP           : {cep_50:.2f} m")
    print(
        f"Error Summary : Average: {average_error:.2f} m, Min: {min_error:.2f} m, Max: {max_error:.2f} m"
    )
    print(
        f"CN0 Summary   : Average: {average_cn0:.2f}, Min: {min_cn0:.2f}, Max: {max_cn0:.2f}"
    )
    print(
        f"HDOP Summary  : Average: {average_hdop:.2f}, Min: {min_hdop:.2f}, Max: {max_hdop:.2f}"
    )

    return test_set, {
        "reference_latitude": reference_latitude,
        "reference_longitude": reference_longitude,
        "cep_50": cep_50,
        "average_error": average_error,
        "min_error": min_error,
        "max_error": max_error,
    }


def plot_gps_scatter(test_set, stats, title="GPS Accuracy Scatter", output_file=None):
    ref_lat = stats["reference_latitude"]
    ref_lon = stats["reference_longitude"]
    cep_50 = stats["cep_50"]

    # Convert lat/lon offsets to approximate metres so axes are in real distance.
    # 1° lat ≈ 111_320 m; 1° lon ≈ 111_320 * cos(lat) m
    lat_scale = 111_320
    lon_scale = 111_320 * np.cos(np.radians(ref_lat))

    dx = (test_set["location_longitude"] - ref_lon) * lon_scale
    dy = (test_set["location_latitude"] - ref_lat) * lat_scale

    fig, ax = plt.subplots(figsize=(7, 7))
    # Light theme
    fig.patch.set_facecolor("white")
    ax.set_facecolor("#f7f7f7")

    # Grid
    ax.grid(color="#cccccc", linewidth=0.6, linestyle="--", alpha=0.8)

    # Scatter – test points coloured by error distance (plasma: yellow→red→purple)
    scatter = ax.scatter(
        dx,
        dy,
        c=test_set["error_distance_m"],
        cmap="plasma",
        s=40,
        alpha=0.9,
        edgecolors="none",
        zorder=3,
        label="Test points",
    )
    cbar = fig.colorbar(scatter, ax=ax, pad=0.02)
    cbar.set_label("Error distance (m)", color="#333333", fontsize=9)
    cbar.ax.yaxis.set_tick_params(color="#333333")
    plt.setp(cbar.ax.yaxis.get_ticklabels(), color="#333333")

    # Reference centre marker
    ax.scatter(
        0,
        0,
        color="#1a7f37",
        s=120,
        zorder=5,
        marker="+",
        linewidths=2.5,
        label=f"Reference ({ref_lat:.6f}, {ref_lon:.6f})",
    )

    # CEP-50 circle — cyan so it never overlaps the plasma (yellow/red/purple) dots
    cep_circle = mpatches.Circle(
        (0, 0),
        cep_50,
        fill=False,
        edgecolor="#00b4d8",
        linewidth=2.0,
        linestyle="--",
        zorder=4,
        label=f"CEP50 = {cep_50:.2f} m",
    )
    ax.add_patch(cep_circle)

    # Padding: 30 % beyond the furthest point from origin
    max_extent = max(dx.abs().max(), dy.abs().max(), cep_50)
    pad = max_extent * 1.60
    ax.set_xlim(-pad, pad)
    ax.set_ylim(-pad, pad)
    ax.set_aspect("equal")

    # Labels & legend
    ax.set_xlabel("East offset (m)", color="#333333")
    ax.set_ylabel("North offset (m)", color="#333333")
    ax.set_title(title, color="#111111", fontsize=13, fontweight="bold", pad=14)
    ax.tick_params(colors="#333333")
    for spine in ax.spines.values():
        spine.set_edgecolor("#aaaaaa")

    ax.legend(
        facecolor="white",
        edgecolor="#aaaaaa",
        labelcolor="#111111",
        fontsize=8.5,
        loc="upper right",
    )

    plt.tight_layout()
    if output_file:
        plt.savefig(output_file, dpi=150, bbox_inches="tight")
        print(f"Plot saved to {output_file}")
    else:
        plt.show()


def _plot_error_vs_metric(
    test_set,
    metric_col,
    metric_label,
    metric_color,
    title,
    output_file,
):
    """Dual-axis line graph: error distance (left) vs a satellite metric (right)."""
    x = test_set["test_id"].values
    error = test_set["error_distance_m"].values
    metric = test_set[metric_col].values

    fig, ax1 = plt.subplots(figsize=(7, 7))
    fig.patch.set_facecolor("white")
    ax1.set_facecolor("#f7f7f7")
    ax1.grid(color="#cccccc", linewidth=0.6, linestyle="--", alpha=0.8, zorder=0)

    # --- Left axis: error distance ---
    error_color = "#e05c00"
    ax1.plot(
        x,
        error,
        color=error_color,
        linewidth=1.4,
        alpha=0.9,
        label="Error distance (m)",
        zorder=2,
    )
    ax1.fill_between(x, error, alpha=0.12, color=error_color, zorder=1)
    ax1.set_xlabel("Test ID", color="#333333")
    ax1.set_ylabel("Error distance (m)", color=error_color)
    ax1.tick_params(axis="y", colors=error_color)
    ax1.tick_params(axis="x", colors="#333333")
    for spine in ax1.spines.values():
        spine.set_edgecolor("#aaaaaa")

    # --- Right axis: metric ---
    ax2 = ax1.twinx()
    ax2.plot(
        x,
        metric,
        color=metric_color,
        linewidth=1.4,
        alpha=0.9,
        label=metric_label,
        zorder=2,
    )
    ax2.fill_between(x, metric, alpha=0.08, color=metric_color, zorder=1)
    ax2.set_ylabel(metric_label, color=metric_color)
    ax2.tick_params(axis="y", colors=metric_color)
    ax2.spines["right"].set_edgecolor(metric_color)

    # Combined legend — placed outside the axes so it never overlaps the lines
    lines1, labels1 = ax1.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax1.legend(
        lines1 + lines2,
        labels1 + labels2,
        facecolor="white",
        edgecolor="#aaaaaa",
        labelcolor="#111111",
        fontsize=8.5,
        loc="lower left",
        bbox_to_anchor=(0, 1.01),
        borderaxespad=0,
        ncols=2,
    )

    ax1.set_title(title, color="#111111", fontsize=13, fontweight="bold", pad=42)
    plt.tight_layout()
    if output_file:
        plt.savefig(output_file, dpi=150, bbox_inches="tight")
        print(f"Plot saved to {output_file}")
    else:
        plt.show()


def plot_error_vs_cn0(test_set, title="Error Distance vs C/N0", output_file=None):
    _plot_error_vs_metric(
        test_set,
        metric_col="satellites_carrier_to_noise",
        metric_label="C/N0 (dB-Hz)",
        metric_color="#0077b6",
        title=title,
        output_file=output_file,
    )


def plot_error_vs_hdop(test_set, title="Error Distance vs HDOP", output_file=None):
    _plot_error_vs_metric(
        test_set,
        metric_col="dop_hdop",
        metric_label="HDOP",
        metric_color="#6a0dad",
        title=title,
        output_file=output_file,
    )


def pengujian_gsm(start_test_id, end_test_id, csv_file):
    df = pd.read_csv(csv_file, sep=";")
    ts = df[(df["test_id"] >= start_test_id) & (df["test_id"] <= end_test_id)].copy()

    # Fill missing RSSI with mean
    if ts["network_rssi"].isna().any():
        mean_rssi = ts["network_rssi"].mean()
        n = ts["network_rssi"].isna().sum()
        ts["network_rssi"] = ts["network_rssi"].fillna(mean_rssi)
        print(
            f"Filled {n} missing value(s) in 'network_rssi' with mean ({mean_rssi:.2f})"
        )

    total = len(ts)
    print(f"Total GSM test rows : {total}")
    print(
        f"RSSI range          : {ts['network_rssi'].min()} – {ts['network_rssi'].max()}"
    )
    print(
        f"Unique cell towers  : {ts['network_ci'].nunique()} ({', '.join(ts['network_ci'].unique())})"
    )
    return ts


def plot_gsm_route(ts, title="GSM Route", output_file=None):
    lats = ts["location_latitude"].values
    lons = ts["location_longitude"].values
    rssi = ts["network_rssi"].values
    cells = ts["network_ci"].values

    fig, ax = plt.subplots(figsize=(7, 7))
    fig.patch.set_facecolor("white")
    ax.set_facecolor("#f7f7f7")
    ax.grid(color="#cccccc", linewidth=0.6, linestyle="--", alpha=0.8)

    # --- Route line coloured by RSSI using LineCollection ---
    import matplotlib.cm as cm
    import matplotlib.colors as mcolors
    from matplotlib.collections import LineCollection

    # Build segments: each segment is [[lon0,lat0],[lon1,lat1]]
    points = np.array([lons, lats]).T.reshape(-1, 1, 2)
    segments = np.concatenate([points[:-1], points[1:]], axis=1)
    rssi_seg = (rssi[:-1] + rssi[1:]) / 2  # colour = avg of the two endpoints

    norm = mcolors.Normalize(vmin=rssi.min(), vmax=rssi.max())
    lc = LineCollection(
        segments, cmap="RdYlGn", norm=norm, linewidth=2.5, zorder=2, alpha=0.9
    )
    lc.set_array(rssi_seg)
    ax.add_collection(lc)

    cbar = fig.colorbar(lc, ax=ax, pad=0.02)
    cbar.set_label("RSSI", color="#333333", fontsize=9)
    cbar.ax.yaxis.set_tick_params(color="#333333")
    plt.setp(cbar.ax.yaxis.get_ticklabels(), color="#333333")

    # --- Start / end markers ---
    ax.scatter(
        lons[0], lats[0], color="#1a7f37", s=100, zorder=5, marker="o", label="Start"
    )
    ax.scatter(
        lons[-1], lats[-1], color="#c0392b", s=100, zorder=5, marker="s", label="End"
    )

    # --- Cell-tower handoff markers ---
    handoff_label_added = False
    for i in range(1, len(cells)):
        if cells[i] != cells[i - 1]:
            label = "Cell Tower Change" if not handoff_label_added else ""
            handoff_label_added = True
            ax.scatter(
                lons[i],
                lats[i],
                color="#8a2be2",
                s=70,
                zorder=6,
                marker="^",
                edgecolors="white",
                linewidths=0.6,
                label=label,
            )

    # --- Axes ---
    ax.set_xlabel("Longitude", color="#333333")
    ax.set_ylabel("Latitude", color="#333333")
    ax.set_title(title, color="#111111", fontsize=13, fontweight="bold", pad=14)
    ax.tick_params(colors="#333333")
    for spine in ax.spines.values():
        spine.set_edgecolor("#aaaaaa")

    ax.autoscale_view()

    ax.legend(
        facecolor="white",
        edgecolor="#aaaaaa",
        labelcolor="#111111",
        fontsize=7.5,
        loc="lower left",
        bbox_to_anchor=(0, 1.01),
        borderaxespad=0,
        ncols=4,
    )
    ax.set_title(title, color="#111111", fontsize=13, fontweight="bold", pad=42)

    plt.tight_layout()
    if output_file:
        plt.savefig(output_file, dpi=150, bbox_inches="tight")
        print(f"Plot saved to {output_file}")
    else:
        plt.show()


def pengujian_qos(start_test_id, end_test_id, csv_file):
    df = pd.read_csv(csv_file, sep=";")
    
    # Filter by test_id
    df = df[(df["test_id"] >= start_test_id) & (df["test_id"] <= end_test_id)].copy()
    
    # 1. Total connection retries
    total_retries = int(df["connection_retries"].sum())
    
    # 2. Sequence/Iteration IDs as they are in the CSV
    total_sequence_count = len(df)
    max_sequence_id = int(df["connection_sequence_id"].max()) if not df["connection_sequence_id"].empty else 0
    min_sequence_id = int(df["connection_sequence_id"].min()) if not df["connection_sequence_id"].empty else 0
    
    # 3. Create latency column
    df["datetime_send"] = pd.to_datetime(df["datetime_iso8601"], errors="coerce", utc=True)
    df["datetime_receive"] = pd.to_datetime(df["received_at"], errors="coerce", utc=True)
    
    is_nan = df["datetime_send"].isna()
    is_1980 = df["datetime_send"].dt.year == 1980
    is_duplicate = df["datetime_send"] == df["datetime_send"].shift(1)
    is_frozen = is_nan | is_1980 | is_duplicate
    
    df["latency"] = (df["datetime_receive"] - df["datetime_send"]).dt.total_seconds()
    
    # Ignore frozen rows (don't remove the row, just keep latency as None/NaN)
    df.loc[is_frozen, "latency"] = None
    
    valid_latencies = df["latency"].dropna()
    avg_latency = valid_latencies.mean() if not valid_latencies.empty else 0
    max_latency = valid_latencies.max() if not valid_latencies.empty else 0
    min_latency = valid_latencies.min() if not valid_latencies.empty else 0
    
    print(f"QoS Analysis (from {csv_file}, test_id {start_test_id} - {end_test_id}):")
    print(f"Connection retries happened              : {total_retries}")
    print(f"Total sequence_id (count of records)     : {total_sequence_count}")
    print(f"Sequence ID range in raw data            : {min_sequence_id} – {max_sequence_id}")
    print(f"Iteration ID different with sequence ID  : {mismatches}")
    print(f"Latency:")
    print(f"  Average                                : {avg_latency:.4f} s")
    print(f"  Maximum                                : {max_latency:.4f} s")
    print(f"  Minimum                                : {min_latency:.4f} s")
    
    return df


def main():
    print("Hello from testing!")


if __name__ == "__main__":
    main()
