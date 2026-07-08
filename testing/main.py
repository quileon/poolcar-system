import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns
from matplotlib.pylab import f
from sklearn.metrics import average_precision_score
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

    return


def main():
    print("Hello from testing!")


if __name__ == "__main__":
    main()
