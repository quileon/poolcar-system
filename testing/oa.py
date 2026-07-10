from main import pengujian_akurasi_gps, plot_error_vs_cn0, plot_error_vs_hdop, plot_gps_scatter

# Open Area
START_TEST_ID = 3601
END_TEST_ID = 3700
REFERENCE_GPS_START_TEST_ID = 3751
REFERENCE_GPS_END_TEST_ID = 3781
CSV_FILE = "test.csv"


def main():
    print("Analyzing OA scenario...\n")
    test_set, stats = pengujian_akurasi_gps(
        START_TEST_ID,
        END_TEST_ID,
        REFERENCE_GPS_START_TEST_ID,
        REFERENCE_GPS_END_TEST_ID,
        CSV_FILE,
    )
    plot_gps_scatter(
        test_set, stats, title="GPS Accuracy – Open Area", output_file="oa_scatter.png"
    )
    plot_error_vs_cn0(
        test_set, title="Error Distance vs C/N0 – Open Area", output_file="oa_error_cn0.png"
    )
    plot_error_vs_hdop(
        test_set, title="Error Distance vs HDOP – Open Area", output_file="oa_error_hdop.png"
    )


if __name__ == "__main__":
    main()
