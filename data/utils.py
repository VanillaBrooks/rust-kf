
import pandas

class Data:
    def __init__(self, x, y):
        self.x_data = x
        self.y_data = y


def setup_data(path, df=False, clip=True):
    data = pandas.read_csv(path)
    if clip:
        data = data.clip(-clip, clip)

    # return the dataframe directly
    if df:
        return data

    x_series = data['x_points'].rename("residual distance from truth x hit")
    y_series = data['y_points'].rename("residual distance from truth y hit")

    return Data(x_series, y_series)
