import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import seaborn as sns
import pandas as pd

def main():

    sns.set(rc={'axes.facecolor':'lightgray'}, font_scale=2)

    filename = "client_watcher"

    # Load the data using pandas
    data = pd.read_csv(f"{filename}.dat", sep=' ', names=['Value'], dtype={'Value': int})

    # Map timestamps to start at 0 and increase by 0.1
    counter = 0
    mapped_timestamps = []

    for timestamp in data['Value']:
        mapped_timestamps.append(counter)
        counter += 1

    data['MappedTimestamp'] = mapped_timestamps

    fig, ax = plt.subplots()

    # Create a plot using seaborn
    # plt.figure(figsize=(10, 6))  # Adjust figure size if needed
    sns.lineplot(data=data, x='MappedTimestamp', y='Value', linewidth=2.5)

    scale_x = 1e6
    ticks_x = ticker.FuncFormatter(lambda x, pos: '{0:g}'.format(x/scale_x))
    ax.yaxis.set_major_formatter(ticks_x)

    # Set title and labels
    plt.title('Memory Usage (Client)', fontsize=28)
    plt.xlabel('Time (s)', fontsize=20)
    plt.ylabel('Memory Usage (MiB)', fontsize=20)
    # plt.ticklabel_format(style='plain', axis='y')

    # Display the plot
    plt.show()
    plt.savefig(f"{filename}.png")

if __name__ == "__main__":
    main()