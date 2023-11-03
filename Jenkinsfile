pipeline {
  agent {
    dockerfile {
      filename 'Dockerfile_ubuntu_2304'
      dir 'tools/build-env'
    }
  }

  environment {
    FULL_VERSION = sh(script: "./tools/get_version.sh full", returnStdout: true).trim()
  }

  stages {
    stage('Download prerequisites') {
      steps {
        dir('ttg') {
          git url: 'https://github.com/maximmenshikov/ttg.git',
              branch: 'main'
        }
      }
    }
    stage('Build for all platforms') {
      parallel {
        stage('Build (Linux)') {
          steps {
            sh 'env PATH=${HOME}/.cargo/bin:${PATH} rustc --version'
          }
        }
      }
    }
  }
  post {
    success {
      sh './ttg/ttg_send_notification --env --ignore-bad -- "${JOB_NAME}/${BUILD_NUMBER}: PASSED"'
    }
    failure {
      sh './ttg/ttg_send_notification --env --ignore-bad -- "${JOB_NAME}/${BUILD_NUMBER}: FAILED. See details in ${BUILD_URL}"'
    }
  }
}
