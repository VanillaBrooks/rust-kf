import json
import os
from multiprocessing import Process

import seaborn as sns
import pandas
import matplotlib.pyplot as plt
from pprint import pprint

from utils import Data, setup_data

current_dir = os.path.dirname(__file__)
hist_folder = os.path.join(current_dir, "histograms")
csv_folder = "E:\\kf_csvs\\"

# produces smoothing / filteres / prediction residuals 
def generate_histogram(info_json, smth, filt, pred):

    std = info_json['stdevs']

    title = f"n={info_json['iterations']}, sensors={info_json['num_sensors']}, sensor distance ={info_json['sensor_distance']} point std = {std['point_std']}, \ndiagonal covariance std={std['diag_std']}, non-diagonal covariance std={std['corner_std']}, \ndiagonal covariance mean = {std['diag_mean']}, non-diagonal mean = {std['corner_mean']}"

    save_name = info_json['histogram_name']

    plt.xticks(rotation=70)
    fig, axes = plt.subplots(3, 2, figsize=(12,12), sharey=True, sharex=False)

    bins = 1000

    fig.suptitle(title, fontsize=15)


    #smoothed data
    axes[0, 0].hist(smth.x_data, bins)
    axes[0, 0].set_title("smoothed x residuals")
    axes[0, 0].tick_params(axis='x', rotation=20)

    axes[0, 1].hist(smth.y_data, bins)
    axes[0, 1].set_title("smoothed y residuals")
    axes[0, 1].tick_params(axis='x', rotation=20)


    # filtered data
    axes[1, 0].hist(filt.x_data, bins)
    axes[1, 0].set_title("filtered x residuals")
    axes[1, 0].tick_params(axis='x', rotation=20)

    axes[1, 1].hist(filt.y_data, bins)
    axes[1, 1].set_title("filtered y residuals")
    axes[1, 1].tick_params(axis='x', rotation=20)


    # predicted data
    axes[2, 0].hist(pred.x_data, bins)
    axes[2, 0].set_title("predicted x residuals")
    axes[1, 1].tick_params(axis='x', rotation=20)

    axes[2, 1].hist(pred.y_data, bins)
    axes[2, 1].set_title("predicted y residuals")
    axes[1, 1].tick_params(axis='x', rotation=20)

    save_loc = os.path.join(hist_folder, save_name)
    plt.savefig(save_loc, dpi=300)


def pull_meta_data(directory):

    sub_folder = os.path.join(csv_folder, directory)
    smth = setup_data(os.path.join(sub_folder, "smth.csv"))
    filt = setup_data(os.path.join(sub_folder, "filt.csv"))
    pred = setup_data(os.path.join(sub_folder, "pred.csv"))

    json_ = os.path.join(sub_folder, "info.json")
    with open (json_, 'r') as f:
        j = json.load(f)
    
    generate_histogram(j, smth, filt, pred)

# difference between truth values and their smeared results
def plot_truth_smear_residuals(path):
    path = os.path.join(path, "smth.csv")

    data = setup_data(path)
    bins = 30

    sns.set_style("darkgrid")

    plt.hist(data.x_data, bins)

    plt.savefig(r"C:\Users\Brooks\github\rust-kf\data\truth_smear_residuals.png")

# scatterplot of prediction residauls
def residual_scatterplot(path):
    path = os.path.join(path, "smth.csv")

    data = setup_data(path, df=True)


    plt.scatter(data['x_points'], data['y_points'], alpha=.05)  # matplotlib version
    # sns.scatterplot("x_points", "y_points", data=data)        # seaborn version

    plt.savefig(r"C:\Users\Brooks\github\rust-kf\data\residual_scatterplot.png")


# multiprocessing to create histograms over the bulk data produced by the KF
def main():

    for i in os.listdir(csv_folder):
        print("process started")
        p = Process(target=pull_meta_data, args=(i,))
        p.start()

# handles histograms and scatterplots of all smoothing / prediction / filter data on
# a per-sensor basis 
def main_sensor_residuals():
    path = r"C:\Users\Brooks\github\rust-kf\data\initial_prediction_data"
    count = 10

    prefixes = ["pred", "filt", "smth"]

    for prefix in prefixes:
        _path = os.path.join(path, prefix)
        sensor_residuals(_path, count, prefix)
        per_sensor_scatterplot(_path, count, prefix)
    

# produces histogram of per-sensor residuals. should generally be called by main_sensor_residuals
def sensor_residuals(path, count, type_string):
    sub_folder = "\\sensor_{}.{}"
    save_string = "\\{}_{}.{}"

    # plots all sensors
    fig, axes = plt.subplots(count, 2, figsize=(10, 30), sharey=True, sharex=True)

    fig.suptitle(f"{type_string} vs truth residuals w/smeared state vec\n (sensors 1-10)", fontsize=25)
    bins = 70

    for i in range(count):
        csv_path = path + sub_folder.format(i, "csv")

        data = setup_data(csv_path, clip=False)                 # clip here

        
        axes[i, 0].hist(data.x_data, bins)
        axes[i, 0].set_title("sensor {}".format(i))
        axes[i, 0].tick_params(axis='x', rotation=20)

        axes[i, 1].hist(data.y_data, bins)
        axes[i, 1].set_title("sensor {} y".format(i))
        axes[i, 1].tick_params(axis='x', rotation=20)
        
    save_path = path + save_string.format("all_sensors",type_string, "png")
    plt.savefig(save_path)

    # first sensor only 

    plt.clf()
    plt.cla()

    fig, axes = plt.subplots(1,2, figsize=(10, 5), sharey=True)
    fig.suptitle(
        f"{type_string} vs truth residuals w/smeared state vec\n [first sensor]", fontsize=15)

            
    axes[0].hist(data.x_data, bins)
    axes[0].set_title("sensor {}".format(1))
    axes[0].tick_params(axis='x', rotation=20)

    axes[1].hist(data.y_data, bins)
    axes[1].set_title("sensor {} y".format(1))
    axes[1].tick_params(axis='x', rotation=20)

    save_path = path + save_string.format("first sensor", type_string,"png")
    plt.savefig(save_path)


# produes per-sensor scatterplots of the residuals between truth points and the 
# smth / filt / pred results of KF output. usually called upstream
def per_sensor_scatterplot(folder_path, count, type_string):
    sub_folder = "\\sensor_{}.{}"
    save_string = "\\{}_{}.{}"

    fig, axes = plt.subplots(count, 1, figsize=(10, 60), sharey=True, sharex=True)
    fig.suptitle(f"{type_string} residuals scatterplot with smeared initial state vector", fontsize=20)


    for i in range(count):
        csv_path = folder_path + sub_folder.format(i, "csv")

        data = setup_data(csv_path, clip = False)                               # clip hee

        axes[i].scatter(data.x_data, data.y_data, alpha = .05)
        axes[i].set_title("sensor {} x vs y residuals".format(i))

    save_path = folder_path + save_string.format("scatterplot", type_string, "png")
    plt.savefig(save_path)


resid_data = r"C:\Users\Brooks\github\rust-kf\data\generated_truth_smear_residuals"
if __name__ == "__main__":
    sns.set_style("darkgrid")
    
    # plot_truth_smear_residuals(r"C:\Users\Brooks\github\rust-kf\data\generated_truth_smear_residuals")
    # residual_scatterplot(r"E:\old kf csvs\1000 iterations variants\scale_sensor_distance_0.0000001")
    # residual_scatterplot(r"C:\Users\Brooks\github\rust-kf\data\generated_truth_smear_residuals")#

    # histograms for every csv in the CSV folder
    # main()

    # truth vs smears
    # plot_truth_smear_residuals(r"C:\Users\Brooks\github\rust-kf\data")

    main_sensor_residuals()