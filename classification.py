# ---
# jupyter:
#   jupytext:
#     formats: ipynb,py:light
#     text_representation:
#       extension: .py
#       format_name: light
#       format_version: '1.5'
#       jupytext_version: 1.11.3
#   kernelspec:
#     display_name: Python 3
#     language: python
#     name: python3
# ---

import pandas as pd
import numpy as np
from matplotlib import pyplot as plt

data  =pd.read_csv("logs/27-06-2021_172946.csv",index_col="TIMESTAMP",parse_dates=True)
data

data[["X","Y","Z"]].iloc[500:5500].plot(figsize=(20,5))

acc = data[["X","Y","Z"]].to_numpy()
acc-=acc.mean(axis=0,keepdims=True)
x = np.linalg.norm(acc,axis=-1)

plt.figure(figsize=(20,5))
plt.plot(acc.sum(axis=-1))

plt.figure(figsize=(20,5))
#plt.plot(x)[:500:5500]
plt.plot(acc)

x = acc.sum(axis=-1)#data["Z"].to_numpy() # X IMU raw values
ms_a = data["MILLIS"].to_numpy() # time stamps in milli seconds from arduino
ms_a_n = ms_a-ms_a.min()
ms_a_n = ms_a_n/1000.0
ms_a_n

constant_rate = np.arange(0,ms_a_n.max(),1/104)
resampled = np.interp(constant_rate,xp=ms_a_n,fp=x)

plt.plot(constant_rate[:150],resampled[:150],label="resampled")
plt.plot(ms_a_n[:500],x[:500],label="orig")
plt.legend()

from scipy import signal

len(resampled)/104, 104/3

f, t, Sxx = signal.spectrogram(resampled,fs=104,nperseg=32,noverlap=0,return_onesided=True,detrend="constant",mode="psd")

# +
plt.pcolormesh(t, f, Sxx, shading='gouraud')

plt.ylabel('Frequency [Hz]')

plt.xlabel('Time [sec]')

plt.show()
# -

x_window  =resampled[:(len(resampled)//32)*32].reshape(-1,32)
x_window -=  x_window.mean(axis=-1,keepdims=True)

spectogram = np.fft.rfft(x_window,norm="ortho",n=32)
spectogram = np.abs(spectogram.real)#(np.conjugate(spectogram)*spectogram).real

(spectogram.max(axis=-1).round(1)>.8)

plt.pcolormesh(
    np.arange(0,len(x_window)),
    np.arange(0,104/2,104/(2*spectogram.shape[1])),
    spectogram.T,
    shading='gouraud'
)
plt.plot((spectogram.max(axis=-1).round(1)>.5).astype(int)*2,c="red")

plt.figure(figsize=(20,10))
plt.plot((data["MILLIS"].to_numpy()-data["MILLIS"].to_numpy().min())/1000,data["X"].to_numpy())
plt.plot(t,Sxx[5]*10)
#plt.plot(t,Sxx[2]*100)


# +
targets = (spectogram.max(axis=-1).round(1)>.5)

net = targets & (np.arange(len(targets))<=36)
border = targets & (np.arange(len(targets))>36)
# -

from sklearn import svm
X = spectogram
y = net.astype(int)+border.astype(int)*2#targets.astype(float)
clf = svm.LinearSVC(class_weight="balanced")
clf.fit(X, y)

clf.score(X,y)

# +
from sklearn.decomposition import PCA

pca = PCA(n_components=2)
pca.fit(X)
x2=pca.transform(X)

plt.figure(figsize=(10,10))

for i,c in enumerate(["no hit","net","rim"]):
    plt.scatter(*(x2[y==i]).T,label=c)

    plt.legend()
wrong = x2[clf.predict(X)!=y]
plt.scatter(*wrong.T,s=50,marker="x",c="red")
# -

clf.predict(X)

x2.min(),x2.max()

a =np.mgrid[0:100,0:100]/100*(x2.max()-x2.min())+x2.min()
a17=pca.inverse_transform(a.T)
pred = clf.predict(a17.reshape(-1,17)).reshape(100,100)
plt.imshow(pred)


