import json
import os
from multiprocessing import Process

import seaborn as sns
import pandas
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
from pprint import pprint

from utils import Data, setup_data

current_dir = os.path.dirname(__file__)


# Produces a 2x2 plot of points in 3d space that are produced from
# RKN, as well as their projections onto each plane
def graph_3d_points(rel_path):
    full_path = current_dir + "\\" + rel_path
    
    df = setup_data(full_path, True, False)

    fig = plt.figure(figsize=(12,12))
    fig.suptitle("global points produced by runge-kutta", fontsize=15)

    # 3d plot of points
    ax = fig.add_subplot(221,projection='3d')
    
    ax.plot(df['x'], df['y'], df['z'])
    ax.set_title("3 dimensional path")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.set_zlabel("z")

    # x,y only (projection onto z)

    ax = fig.add_subplot(222)

    ax.set_title("projection to z-plane")
    ax.plot(df['x'], df['y'])
    ax.set_xlabel("x")
    ax.set_ylabel("y")

    # x, z only (proj onto y axis)

    ax = fig.add_subplot(223)
    ax.plot(df['x'], df['z'])
    ax.set_title("projection to y-plane")
    ax.set_xlabel("x")
    ax.set_ylabel("z")

    # z y only (proj to x axis)

    ax = fig.add_subplot(224)
    ax.plot(df['y'], df['z'])
    ax.set_title("projection to x-plane")
    ax.set_xlabel("y")
    ax.set_ylabel("z")

    save_folder = current_dir + "\\" + "runge_kutta_global_points.png"
    plt.savefig(save_folder, dpi=300)
    

def runge_kutta_residuals(rel_path):
    full_path = current_dir + "\\" + rel_path
    
    df = setup_data(full_path, True, False)

    bins = 60

    fig = plt.figure(figsize=(12,12))

    fig.suptitle("residuals after multiple steps of runge-kutta")

    ax = fig.add_subplot(221)
    ax.hist(df['x_points'], bins)
    ax.set_title("x points")
    
    ax = fig.add_subplot(222)
    ax.hist(df['y_points'], bins)
    ax.set_title("y points")

    ax = fig.add_subplot(223)
    ax.scatter(df['x_points'], df["y_points"], alpha=0.05)
    ax.set_title("x residual vs y residual")

    save_folder = current_dir + "\\" + "runge_kutta_points_after_iteration.png"

    plt.savefig(save_folder,dpi=300)



if __name__ == "__main__":
    # graph_3d_points("\\rk_points.csv")
    runge_kutta_residuals("\\runge_kutta_truth_smear_residuals.csv")
