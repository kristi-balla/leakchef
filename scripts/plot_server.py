import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import seaborn as sns
import pandas as pd

def main():

    sns.set(rc={'axes.facecolor':'lightgray'}, font_scale=2)

    files = ["server_watcher_1c.dat", "server_watcher_2c.dat", "server_watcher_4c.dat", "server_watcher_8c.dat", "server_watcher_16c.dat"]

    # Create an empty DataFrame
    df = pd.DataFrame()

    # Read data from each .dat file and add it as a column in the DataFrame
    for i, file in enumerate(files):
        column_name = f"{2**i}C"
        data = pd.read_csv(file, header=None, names=[column_name])
        df = pd.concat([df, data], axis=1)

    # Map timestamps to start at 0 and increase by 0.1
    # counter = 0
    # mapped_timestamps = []

    # for timestamp in data['16C']:
    #     mapped_timestamps.append(counter)
    #     counter += 1

    # data['MappedTimestamp'] = mapped_timestamps

    # Create a line plot using Seaborn
    # sns.set(style="whitegrid")  # Optional, for grid lines
    fig, ax = plt.subplots()

    sns.set_palette("colorblind")
    sns.lineplot(data=df)

    scale_x = 1e6
    ticks_x = ticker.FuncFormatter(lambda x, pos: '{0:g}'.format(x/scale_x))
    ax.yaxis.set_major_formatter(ticks_x)

    # Add labels and title
    plt.xlabel("Time (s)", fontsize=20)
    plt.ylabel("Memory Usage (MiB)", fontsize=20)
    plt.title("Memory Usage (Server)", fontsize=28)

    # Show the plot
    plt.legend()  # Optional, adds a legend
    plt.show()
    plt.savefig("server.png")

    ##########################################################################################################

    # # Load the data using pandas
    # data = pd.read_csv(f"{filename}.dat", sep=' ', names=['Value'], dtype={'Value': int})

    # # Map timestamps to start at 0 and increase by 0.1
    # counter = 0
    # mapped_timestamps = []

    # for timestamp in data['Value']:
    #     mapped_timestamps.append(counter)
    #     counter += 1

    # data['MappedTimestamp'] = mapped_timestamps

    # fig, ax = plt.subplots()

    # # Create a plot using seaborn
    # # plt.figure(figsize=(10, 6))  # Adjust figure size if needed
    # sns.lineplot(data=data, x='MappedTimestamp', y='Value', linewidth=2.5)

    # scale_x = 1e6
    # ticks_x = ticker.FuncFormatter(lambda x, pos: '{0:g}'.format(x/scale_x))
    # ax.yaxis.set_major_formatter(ticks_x)

    # # Set title and labels
    # plt.title('Memory Usage Over Time', fontsize=20)
    # plt.xlabel('Time (s)', fontsize=16)
    # plt.ylabel('Memory Usage (MiB)', fontsize=16)
    # # plt.ticklabel_format(style='plain', axis='y')

    # # Display the plot
    # plt.show()
    # plt.savefig(f"{filename}.png")

if __name__ == "__main__":
    main()