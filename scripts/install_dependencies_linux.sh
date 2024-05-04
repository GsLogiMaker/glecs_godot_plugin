
# Get user password
zenity --password --title="Install Dev Dependencies" | sudo -S echo ""

# General build tools
sudo apt install gcc make -y
if [ $? -ne 0 ]; then # Propogate error
	exit 1
fi

# For Linux 32-bit
sudo apt install gcc-multilib -y
if [ $? -ne 0 ]; then # Propogate error
	exit 1
fi

# # For Linux ARM 64-bit
# sudo apt install gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu -y
# if [ $? -ne 0 ]; then # Propogate error
# 	exit 1
# fi

# # For Linux ARM 32-bit
# sudo apt install gcc-arm-linux-gnueabi binutils-arm-linux-gnueabi -y
# if [ $? -ne 0 ]; then # Propogate error
# 	exit 1
# fi

# For Windows 64-bit
sudo apt install gcc-mingw-w64 -y
if [ $? -ne 0 ]; then # Propogate error
	exit 1
fi