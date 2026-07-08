from main import pengujian_gsm, plot_gsm_route

# GSM Test
START_TEST_ID = 4775
END_TEST_ID = 4992
CSV_FILE = "test.csv"


def main():
    print("Analyzing GSM scenario...\n")
    ts = pengujian_gsm(START_TEST_ID, END_TEST_ID, CSV_FILE)
    plot_gsm_route(
        ts,
        title="GSM Strength",
        output_file="gsm_route.png",
    )


if __name__ == "__main__":
    main()
