import seaborn as sns
import matplotlib.pyplot as plt

sns.set(rc={'axes.facecolor':'lightgray'})

# Your data
data = {"small": 1304.89, "medium": 770.43, "large": 1068.70}

# Create a bar plot
sns.barplot(x=list(data.keys()), y=list(data.values()))

# Add labels and title
plt.xlabel("leak_size", fontsize=16)
plt.ylabel("Time (s)", fontsize=16)
plt.title("RTT for different leak sizes", fontsize=20)

# Show the plot
plt.show()
plt.savefig("rtt.png")