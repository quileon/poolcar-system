from main import pengujian_qos

# QoS Test Scenario
START_TEST_ID = 1669
END_TEST_ID = 5449
CSV_FILE = "test2.csv"


def main():
    print("Analyzing QoS scenario...\n")
    pengujian_qos(START_TEST_ID, END_TEST_ID, CSV_FILE)


if __name__ == "__main__":
    main()
