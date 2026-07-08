from datetime import datetime

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

# Configuration
START_TEST_ID = 1669
END_TEST_ID = 5444
CSV_FILE = "test2.csv"


def analyze_qos(start_test_id, end_test_id, csv_file):
    """
    Comprehensive QoS Analysis:
    - Retries: count from connection_retries field
    - Frozen datetimes: detect and exclude (1980 dates, duplicates)
    - Latency: datetime_receive - datetime_send (excluding frozen)
    - Packet Loss: when connection_sequence_id iteration is skipped
    """

    print("=" * 80)
    print("QoS ANALYSIS")
    print("=" * 80)

    # Load data
    df = pd.read_csv(csv_file, sep=";")

    # Filter by test_id
    df = df[(df["test_id"] >= start_test_id) & (df["test_id"] <= end_test_id)].copy()
    df = df.reset_index(drop=True)

    print(
        f"\n[INFO] Loaded {len(df)} records (test_id: {start_test_id} - {end_test_id})"
    )

    # =========================================================================
    # 1. RETRIES ANALYSIS
    # =========================================================================
    total_retries = int(df["connection_retries"].sum())
    print(f"\n[RETRIES]")
    print(f"  Total connection retries: {total_retries}")

    # Count retry events (when sequence_id doesn't increase but iteration_id does)
    retry_events = 0
    sequence_id_prev = None
    iteration_id_prev = None

    for idx, row in df.iterrows():
        seq_id = row["connection_sequence_id"]
        iter_id = row["connection_iteration_id"]

        if sequence_id_prev is not None:
            # Retry detected: sequence_id stays same, iteration_id increases
            if seq_id == sequence_id_prev and iter_id > iteration_id_prev:
                retry_events += 1

        sequence_id_prev = seq_id
        iteration_id_prev = iter_id

    print(f"  Retry events (seq_id same, iter_id increases): {retry_events}")

    # =========================================================================
    # 2. DATETIME HANDLING (Frozen Datetimes)
    # =========================================================================
    df["datetime_send"] = pd.to_datetime(
        df["datetime_iso8601"], errors="coerce", utc=True
    )
    df["datetime_receive"] = pd.to_datetime(
        df["received_at"], errors="coerce", utc=True
    )

    # Detect frozen datetimes
    is_nan = df["datetime_send"].isna()
    is_1980 = df["datetime_send"].dt.year == 1980
    is_duplicate = df["datetime_send"] == df["datetime_send"].shift(1)
    is_frozen = is_nan | is_1980 | is_duplicate

    frozen_count = is_frozen.sum()
    print(f"\n[FROZEN DATETIMES]")
    print(f"  NaN datetime: {is_nan.sum()}")
    print(f"  Year 1980: {is_1980.sum()}")
    print(f"  Duplicate (frozen iteration): {is_duplicate.sum()}")
    print(f"  Total frozen: {frozen_count}")

    # =========================================================================
    # 3. LATENCY CALCULATION
    # =========================================================================
    df["latency_seconds"] = (
        df["datetime_receive"] - df["datetime_send"]
    ).dt.total_seconds()

    # Only use valid (non-frozen) latencies
    df.loc[is_frozen, "latency_seconds"] = np.nan

    valid_latencies = df["latency_seconds"].dropna()

    if not valid_latencies.empty:
        min_latency = valid_latencies.min()
        max_latency = valid_latencies.max()
        avg_latency = valid_latencies.mean()
        median_latency = valid_latencies.median()
    else:
        min_latency = max_latency = avg_latency = median_latency = 0

    print(f"\n[LATENCY (seconds, excluding frozen datetimes)]")
    print(f"  Valid latency records: {len(valid_latencies)}")
    print(f"  Minimum: {min_latency:.4f}s")
    print(f"  Maximum: {max_latency:.4f}s")
    print(f"  Average: {avg_latency:.4f}s")
    print(f"  Median: {median_latency:.4f}s")

    # =========================================================================
    # 4. PACKET LOSS ANALYSIS
    # =========================================================================
    print(f"\n[PACKET LOSS]")

    # Detect packet loss: when sequence_id skips
    packet_loss_events = []
    seq_ids = df["connection_sequence_id"].values
    test_ids = df["test_id"].values

    for i in range(1, len(seq_ids)):
        if seq_ids[i] != seq_ids[i - 1]:  # Sequence changed
            expected_next = seq_ids[i - 1] + 1
            if seq_ids[i] > expected_next:
                # Packet loss: sequence jumped
                loss_count = seq_ids[i] - seq_ids[i - 1] - 1
                packet_loss_events.append(
                    {
                        "test_id": test_ids[i],
                        "from_seq": seq_ids[i - 1],
                        "to_seq": seq_ids[i],
                        "lost_count": loss_count,
                    }
                )

    total_loss_count = sum([abs(e["lost_count"]) for e in packet_loss_events])
    print(f"  Packet loss events: {len(packet_loss_events)}")
    print(f"  Total packets lost: {total_loss_count}")

    if len(packet_loss_events) > 0:
        print(f"  First few loss events:")
        for event in packet_loss_events[:5]:
            print(
                f"    test_id={event['test_id']}: seq {event['from_seq']} → {event['to_seq']} (lost {event['lost_count']})"
            )

    print("\n" + "=" * 80)

    return {
        "df": df,
        "total_retries": total_retries,
        "retry_events": retry_events,
        "frozen_count": frozen_count,
        "min_latency": min_latency,
        "max_latency": max_latency,
        "avg_latency": avg_latency,
        "median_latency": median_latency,
        "packet_loss_events": packet_loss_events,
        "total_loss_count": total_loss_count,
        "valid_latencies": valid_latencies,
    }


def plot_sequence_id_with_loss(
    df, packet_loss_events, test_ids, output_file="qos_sequence_loss.png"
):
    """
    Plot sequence_id over test_id with red X markers for packet loss events.
    Similar to the reference image (qos1.png).
    """
    fig, ax = plt.subplots(figsize=(12, 5))
    fig.patch.set_facecolor("white")
    ax.set_facecolor("white")

    # Plot sequence_id as a line
    ax.plot(
        test_ids,
        df["connection_sequence_id"].values,
        color="#6ba3ff",
        linewidth=2,
        label="Sequence ID",
        zorder=2,
    )

    # Mark packet loss events with red X
    if len(packet_loss_events) > 0:
        loss_test_ids = [e["test_id"] for e in packet_loss_events]
        loss_seq_ids = [
            df[df["test_id"] == tid]["connection_sequence_id"].values[0]
            if tid in df["test_id"].values
            else 0
            for tid in loss_test_ids
        ]

        ax.scatter(
            loss_test_ids,
            loss_seq_ids,
            color="red",
            marker="x",
            s=100,
            linewidths=2.5,
            zorder=5,
            label="Packet Loss",
        )

    # Grid and labels
    ax.grid(True, color="#cccccc", linewidth=0.6, alpha=0.7, zorder=1)
    ax.set_xlabel("Test ID", fontsize=11, color="#333333")
    ax.set_ylabel("Sequence ID", fontsize=11, color="#333333")
    ax.set_title(
        "Sequence ID with Packet Loss Events", fontsize=13, fontweight="bold", pad=15
    )
    ax.tick_params(colors="#333333")

    for spine in ax.spines.values():
        spine.set_edgecolor("#999999")
        spine.set_linewidth(1)

    if len(packet_loss_events) > 0:
        ax.legend(fontsize=10, loc="upper left", framealpha=0.95, edgecolor="#999999")

    plt.tight_layout()
    plt.savefig(output_file, dpi=150, bbox_inches="tight")
    print(f"\n✓ Saved: {output_file}")
    plt.close()


def plot_latency_over_time(
    df, valid_latencies, test_ids, output_file="qos_latency.png"
):
    """
    Plot latency over test_id with mean and median lines.
    Similar to the reference image (qos2.png).
    """
    fig, ax = plt.subplots(figsize=(12, 5))
    fig.patch.set_facecolor("white")
    ax.set_facecolor("white")

    # Scatter plot for latencies
    latencies = df["latency_seconds"].values
    ax.scatter(
        test_ids, latencies, color="#7eb3d4", s=30, alpha=0.7, zorder=3, label="Latency"
    )

    # Mean and median lines
    mean_latency = valid_latencies.mean()
    median_latency = valid_latencies.median()

    ax.axhline(
        y=mean_latency,
        color="red",
        linestyle="--",
        linewidth=2,
        label=f"Mean: {mean_latency:.2f}s",
        zorder=2,
    )
    ax.axhline(
        y=median_latency,
        color="orange",
        linestyle="--",
        linewidth=2,
        label=f"Median: {median_latency:.2f}s",
        zorder=2,
    )

    # Grid and labels
    ax.grid(True, color="#cccccc", linewidth=0.6, alpha=0.7, zorder=1)
    ax.set_xlabel("Test ID", fontsize=11, color="#333333")
    ax.set_ylabel("Latency (seconds)", fontsize=11, color="#333333")
    ax.set_title(
        "Latency Over Time (excluding frozen datetimes)",
        fontsize=13,
        fontweight="bold",
        pad=15,
    )
    ax.tick_params(colors="#333333")

    for spine in ax.spines.values():
        spine.set_edgecolor("#999999")
        spine.set_linewidth(1)

    ax.legend(fontsize=10, loc="upper left", framealpha=0.95, edgecolor="#999999")

    plt.tight_layout()
    plt.savefig(output_file, dpi=150, bbox_inches="tight")
    print(f"✓ Saved: {output_file}")
    plt.close()


def main():
    # Run QoS analysis
    results = analyze_qos(START_TEST_ID, END_TEST_ID, CSV_FILE)

    # Get test_ids for plotting
    df = results["df"]
    test_ids = df["test_id"].values

    # Generate plots
    print("\n[GENERATING PLOTS]")
    plot_sequence_id_with_loss(
        df, results["packet_loss_events"], test_ids, output_file="qos_sequence_loss.png"
    )
    plot_latency_over_time(
        df, results["valid_latencies"], test_ids, output_file="qos_latency.png"
    )

    print("\n✓ Analysis complete!")


if __name__ == "__main__":
    main()
