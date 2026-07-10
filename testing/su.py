from main import pengujian_akurasi_gps, plot_error_vs_cn0, plot_error_vs_hdop, plot_gps_scatter

# Semi Urban
START_TEST_ID = 4101
END_TEST_ID = 4200
REFERENCE_GPS_START_TEST_ID = 4251
REFERENCE_GPS_END_TEST_ID = 4300
CSV_FILE = "test.csv"


def main():
    print("Analyzing SU scenario...\n")
    test_set, stats = pengujian_akurasi_gps(
        START_TEST_ID,
        END_TEST_ID,
        REFERENCE_GPS_START_TEST_ID,
        REFERENCE_GPS_END_TEST_ID,
        CSV_FILE,
    )
    plot_gps_scatter(
        test_set, stats, title="GPS Accuracy – Semi Urban", output_file="su_scatter.png"
    )
    plot_error_vs_cn0(
        test_set, title="Error Distance vs C/N0 – Semi Urban", output_file="su_error_cn0.png"
    )
    plot_error_vs_hdop(
        test_set, title="Error Distance vs HDOP – Semi Urban", output_file="su_error_hdop.png"
    )


if __name__ == "__main__":
    main()
