import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns
from matplotlib.patches import Circle
from sklearn.metrics.pairwise import haversine_distances

pd.set_option("display.max_columns", None)
pd.set_option("display.width", None)

def load_and_preprocess_data(filepath: str) -> pd.DataFrame:
    """Loads the CSV and creates datetime and numeric columns."""
    df = pd.read_csv(filepath, sep=";")
    df["datetime_hardware"] = pd.to_datetime(df["datetime_iso8601"], utc=True)
    df["datetime_backend"] = pd.to_datetime(df["received_at"], utc=True)
    df["location_latitude"] = pd.to_numeric(df["location_latitude"])
    df["location_longitude"] = pd.to_numeric(df["location_longitude"])
    return df

def process_gps_data(
    df: pd.DataFrame, 
    start_time_str: str, 
    target_lat: float, 
    target_lon: float
) -> pd.DataFrame:
    """Filters data for 10 minutes from start_time and calculates distance errors."""
    # Define start and end times in local timezone, then convert to UTC
    start_time = pd.Timestamp(start_time_str).tz_convert("UTC")
    end_time = start_time + pd.Timedelta(minutes=10)
    
    # Filter by time
    filtered_df = df[df["datetime_hardware"].between(start_time, end_time)]

    # Select specific columns
    columns_to_keep = [
        "test_id", "location_latitude", "location_longitude",
        "datetime_hardware", "datetime_backend",
        "satellites_visible", "satellites_used", "satellites_carrier_to_noise",
        "dop_hdop", "dop_pdop", "dop_vdop",
    ]
    final_df = filtered_df[columns_to_keep].copy()

    # Add error distances in meters
    target_coords_rad = np.radians([target_lat, target_lon])
    hardware_coords = final_df[["location_latitude", "location_longitude"]].dropna()
    hardware_coords_rad = np.radians(hardware_coords)
    
    error_distances = haversine_distances(hardware_coords_rad, [target_coords_rad]) * 6371000
    error_distances_series = pd.Series(error_distances.flatten(), index=hardware_coords.index)
    
    final_df.insert(len(final_df.columns), "error_distance", error_distances_series)
    return final_df

def plot_gps_drift(final_df: pd.DataFrame, target_lat: float, target_lon: float, save_path: str = None):
    """Generates the CEP scatter plot and Error Distance line graph side-by-side."""
    # Approximate meters per degree at the specific latitude
    meters_per_deg_lat = 111132.92
    meters_per_deg_lon = 111132.92 * np.cos(np.radians(target_lat))

    plot_df = final_df.copy()
    plot_df["x_meters"] = (plot_df["location_longitude"] - target_lon) * meters_per_deg_lon
    plot_df["y_meters"] = (plot_df["location_latitude"] - target_lat) * meters_per_deg_lat

    cep_50 = plot_df["error_distance"].quantile(0.50)
    average_error = plot_df["error_distance"].mean()
    min_error = plot_df["error_distance"].min()
    max_error = plot_df["error_distance"].max()
    
    average_cn0 = plot_df["satellites_carrier_to_noise"].mean()
    min_cn0 = plot_df["satellites_carrier_to_noise"].min()
    max_cn0 = plot_df["satellites_carrier_to_noise"].max()
    
    average_hdop = plot_df["dop_hdop"].mean()
    min_hdop = plot_df["dop_hdop"].min()
    max_hdop = plot_df["dop_hdop"].max()
    
    total_data = len(plot_df)

    print("\n" + "="*40)
    print(f"Total Data Points : {total_data}")
    print("-" * 40)
    print(f"Lowest Error      : {min_error:.2f} meters")
    print(f"Highest Error     : {max_error:.2f} meters")
    print(f"Average Error     : {average_error:.2f} meters")
    print(f"CEP (50%) Error   : {cep_50:.2f} meters")
    print("-" * 40)
    print(f"Lowest C/N0       : {min_cn0:.2f}")
    print(f"Highest C/N0      : {max_cn0:.2f}")
    print(f"Average C/N0      : {average_cn0:.2f}")
    print("-" * 40)
    print(f"Lowest HDOP       : {min_hdop:.2f}")
    print(f"Highest HDOP      : {max_hdop:.2f}")
    print(f"Average HDOP      : {average_hdop:.2f}")
    print("="*40 + "\n")

    sns.set_theme(style="whitegrid")
    fig, (ax1, ax2, ax4) = plt.subplots(1, 3, figsize=(24, 8))

    # ==========================================
    # Subplot 1: Scatter Plot (GPS Drift Map)
    # ==========================================
    # Plot SIM808 coordinates (meters from center)
    sns.scatterplot(
        data=plot_df, x="x_meters", y="y_meters", 
        color="orange", alpha=0.6, label="SIM808 Readings", zorder=3, ax=ax1
    )

    # Plot Ground Truth
    ax1.scatter(0, 0, color="blue", s=150, zorder=5, label="Ground Truth (0, 0)")

    # Plot CEP (50%) Circle
    circle_cep = Circle((0, 0), cep_50, color='green', fill=False, linestyle='-', 
                        linewidth=2, label=f'CEP 50% ({cep_50:.1f} m)', zorder=4)
    ax1.add_patch(circle_cep)

    max_bound = max(plot_df["x_meters"].abs().max(), plot_df["y_meters"].abs().max(), cep_50) * 1.2
    ax1.set_xlim(-max_bound, max_bound)
    ax1.set_ylim(-max_bound, max_bound)

    ax1.set_title("GPS Hardware Drift (Meters from Ground Truth)")
    ax1.set_xlabel("Meters East/West")
    ax1.set_ylabel("Meters North/South")
    ax1.legend(loc='upper right')
    ax1.set_aspect('equal', 'box')

    # ==========================================
    # Subplot 2: Line Graph (Error and C/N0 over Time)
    # ==========================================
    plot_df["elapsed_seconds"] = (plot_df["datetime_hardware"] - plot_df["datetime_hardware"].min()).dt.total_seconds()

    # Plot Error Distance on the primary y-axis
    sns.lineplot(
        data=plot_df, x="elapsed_seconds", y="error_distance", 
        color="red", ax=ax2, marker="o", markersize=4, label="Error Distance"
    )

    ax2.set_title("Error Distance and C/N0 Over Time")
    ax2.set_xlabel("Elapsed Time (Seconds)")
    ax2.set_ylabel("Error Distance (Meters)", color="red")
    ax2.tick_params(axis='y', labelcolor="red")
    
    max_error_val = plot_df["error_distance"].max() * 1.2
    if max_error_val > 0:
        ax2.set_ylim(0, max_error_val)
    
    # Create a twin axis for C/N0
    ax3 = ax2.twinx()
    sns.lineplot(
        data=plot_df, x="elapsed_seconds", y="satellites_carrier_to_noise", 
        color="blue", ax=ax3, marker="s", markersize=4, label="C/N0"
    )
    
    ax3.set_ylabel("Carrier-to-Noise Ratio (C/N0)", color="blue")
    ax3.tick_params(axis='y', labelcolor="blue")
    
    max_cn0_val = plot_df["satellites_carrier_to_noise"].max() * 1.2
    if max_cn0_val > 0:
        ax3.set_ylim(0, max_cn0_val)

    # Combine legends
    lines_1, labels_1 = ax2.get_legend_handles_labels()
    lines_2, labels_2 = ax3.get_legend_handles_labels()
    ax2.legend(lines_1 + lines_2, labels_1 + labels_2, loc="upper right")
    if ax3.get_legend() is not None:
        ax3.get_legend().remove()

    ax2.tick_params(axis='x')

    # ==========================================
    # Subplot 3: Line Graph (Error and HDOP over Time)
    # ==========================================
    # Plot Error Distance on the primary y-axis
    sns.lineplot(
        data=plot_df, x="elapsed_seconds", y="error_distance", 
        color="red", ax=ax4, marker="o", markersize=4, label="Error Distance"
    )

    ax4.set_title("Error Distance and HDOP Over Time")
    ax4.set_xlabel("Elapsed Time (Seconds)")
    ax4.set_ylabel("Error Distance (Meters)", color="red")
    ax4.tick_params(axis='y', labelcolor="red")
    
    if max_error_val > 0:
        ax4.set_ylim(0, max_error_val)
    
    # Create a twin axis for HDOP
    ax5 = ax4.twinx()
    sns.lineplot(
        data=plot_df, x="elapsed_seconds", y="dop_hdop", 
        color="purple", ax=ax5, marker="^", markersize=4, label="HDOP"
    )
    
    ax5.set_ylabel("Horizontal Dilution of Precision (HDOP)", color="purple")
    ax5.tick_params(axis='y', labelcolor="purple")
    
    max_hdop_val = plot_df["dop_hdop"].max() * 1.2
    if max_hdop_val > 0:
        ax5.set_ylim(0, max_hdop_val)

    # Combine legends
    lines_4, labels_4 = ax4.get_legend_handles_labels()
    lines_5, labels_5 = ax5.get_legend_handles_labels()
    ax4.legend(lines_4 + lines_5, labels_4 + labels_5, loc="upper right")
    if ax5.get_legend() is not None:
        ax5.get_legend().remove()

    ax4.tick_params(axis='x')

    plt.tight_layout()
    if save_path:
        plt.savefig(save_path, dpi=300, bbox_inches='tight')
        print(f"Plot saved to: {save_path}")
    plt.show()

def process_gprs_data(df: pd.DataFrame, start_test_id: int, end_test_id: int) -> pd.DataFrame:
    """Filters data by test_id range and calculates latency for GPRS signal test."""
    # Ensure test_id is numeric
    df["test_id"] = pd.to_numeric(df["test_id"], errors='coerce')
    
    filtered_df = df[(df["test_id"] >= start_test_id) & (df["test_id"] <= end_test_id)].copy()
    
    # Ensure RSSI is numeric
    filtered_df["network_rssi"] = pd.to_numeric(filtered_df["network_rssi"], errors='coerce')
    
    # Calculate Latency (Interval between consecutive backend receives)
    filtered_df = filtered_df.sort_values("test_id")
    filtered_df["latency_seconds"] = filtered_df["datetime_backend"].diff().dt.total_seconds().fillna(0)
    
    # Calculate elapsed time for X-axis
    filtered_df["elapsed_seconds"] = (filtered_df["datetime_hardware"] - filtered_df["datetime_hardware"].min()).dt.total_seconds()
    
    # Reset index to start from 1
    filtered_df["reading_index"] = range(1, len(filtered_df) + 1)
    
    return filtered_df

def plot_gprs_signal(plot_df: pd.DataFrame):
    """Generates a dual-axis line graph for RSSI and Latency over time."""
    
    # Calculate stats
    min_rssi = plot_df["network_rssi"].min()
    max_rssi = plot_df["network_rssi"].max()
    avg_rssi = plot_df["network_rssi"].mean()
    
    # Exclude the first 0 latency row from stats for accuracy
    valid_latency = plot_df["latency_seconds"][1:]
    min_latency = valid_latency.min()
    max_latency = valid_latency.max()
    avg_latency = valid_latency.mean()
    
    total_data = len(plot_df)

    print("\n" + "="*40)
    print(f"Total Data Points : {total_data}")
    print(f"Lowest CSQ        : {min_rssi}")
    print(f"Highest CSQ       : {max_rssi}")
    print(f"Average CSQ       : {avg_rssi:.2f}")
    print("-" * 40)
    print(f"Lowest Latency    : {min_latency:.2f} s")
    print(f"Highest Latency   : {max_latency:.2f} s")
    print(f"Average Latency   : {avg_latency:.2f} s")
    print("="*40 + "\n")

    sns.set_theme(style="whitegrid")
    fig, ax1 = plt.subplots(figsize=(12, 6))

    # Plot RSSI
    sns.lineplot(
        data=plot_df, x="reading_index", y="network_rssi", 
        color="blue", ax=ax1, marker="o", markersize=4, label="CSQ"
    )
    
    ax1.set_title("Network CSQ and Latency")
    ax1.set_xlabel("Data Index")
    ax1.set_ylabel("CSQ", color="blue")
    ax1.tick_params(axis='y', labelcolor="blue")
    
    # Plot Latency
    ax2 = ax1.twinx()
    sns.lineplot(
        data=plot_df, x="reading_index", y="latency_seconds", 
        color="red", ax=ax2, marker="s", markersize=4, label="Latency (Seconds)"
    )
    
    ax2.set_ylabel("Latency (Seconds)", color="red")
    ax2.tick_params(axis='y', labelcolor="red")
    
    # Combine legends
    lines_1, labels_1 = ax1.get_legend_handles_labels()
    lines_2, labels_2 = ax2.get_legend_handles_labels()
    ax1.legend(lines_1 + lines_2, labels_1 + labels_2, loc="upper right")
    if ax2.get_legend() is not None:
        ax2.get_legend().remove()

    plt.tight_layout()
    plt.show()

if __name__ == "__main__":
    print("This module only contains core GPS processing and plotting utilities.")