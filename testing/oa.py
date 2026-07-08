from main import pengujian_akurasi_gps

# Open Area
START_TEST_ID = 3601
END_TEST_ID = 3700
REFERENCE_GPS_START_TEST_ID = 3751
REFERENCE_GPS_END_TEST_ID = 3781
CSV_FILE = "test.csv"


def main():
    print("Analyzing OA scenario...\n")
    pengujian_akurasi_gps(
        START_TEST_ID,
        END_TEST_ID,
        REFERENCE_GPS_START_TEST_ID,
        REFERENCE_GPS_END_TEST_ID,
        CSV_FILE,
    )


if __name__ == "__main__":
    main()
