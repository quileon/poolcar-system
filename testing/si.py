from main import pengujian_akurasi_gps, plot_error_vs_cn0, plot_error_vs_hdop, plot_gps_scatter

# Semi Indoor
START_TEST_ID = 4401
END_TEST_ID = 4500
REFERENCE_GPS_START_TEST_ID = 4651
REFERENCE_GPS_END_TEST_ID = 4700
CSV_FILE = "test.csv"


def main():
    print("Analyzing SI scenario...\n")
    test_set, stats = pengujian_akurasi_gps(
        START_TEST_ID,
        END_TEST_ID,
        REFERENCE_GPS_START_TEST_ID,
        REFERENCE_GPS_END_TEST_ID,
        CSV_FILE,
    )
    plot_gps_scatter(
        test_set,
        stats,
        title="GPS Accuracy – Semi Indoor",
        output_file="si_scatter.png",
    )
    plot_error_vs_cn0(
        test_set, title="Error Distance vs C/N0 – Semi Indoor", output_file="si_error_cn0.png"
    )
    plot_error_vs_hdop(
        test_set, title="Error Distance vs HDOP – Semi Indoor", output_file="si_error_hdop.png"
    )


if __name__ == "__main__":
    main()
