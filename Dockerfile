# Dev environment for Git-ai

# Specify the base image
FROM rust:latest
# Add metadata to the image
LABEL maintainer="your_email@example.com"
LABEL version="1.0"
LABEL description="A dev container to house git commands for development"
# Set environment variables
ENV APP_HOME=/usr/app
ENV APP_PORT=8080
# Set the working directory
WORKDIR $APP_HOME
# Copy application files into the container
COPY . .
# Install dependencies
RUN apt-get update && apt-get install -y \
   pkg-config \
   libssl-dev \
   mold \
   clang \
   build-essential && \
   apt-get clean

# Expose the application port
EXPOSE $APP_PORT
# Define the default command to run the application
CMD ["cargo", "build"]