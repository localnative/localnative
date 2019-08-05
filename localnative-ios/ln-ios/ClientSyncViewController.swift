/*
 Local Native
 Copyright (C) 2019  Yi Wang
 
 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU Affero General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.
 
 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU Affero General Public License for more details.
 
 You should have received a copy of the GNU Affero General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
//
//  ClientSyncViewController.swift
//  localnative-ios
//
//  Created by Yi Wang on 8/4/19.
//

import Foundation
import AVFoundation
import UIKit

class ClientSyncViewController : UIViewController, AVCaptureMetadataOutputObjectsDelegate{
    var captureSession: AVCaptureSession!
    var previewLayer: AVCaptureVideoPreviewLayer!

    @IBOutlet weak var cancelButton: UIBarButtonItem!
    @IBOutlet weak var scanButton: UIBarButtonItem!
    @IBOutlet weak var syncButton: UIBarButtonItem!
    @IBOutlet weak var qrView: UIView!
    
    @IBOutlet weak var addrText: UITextView!
    @IBAction func scanTouchDown(_ sender: Any) {
        scan()
    }
    
    @IBAction func cancelTouchDown(_ sender: Any) {
        dismiss(animated: true, completion: nil)
    }
    
    @IBAction func syncTouchDown(_ sender: Any) {
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()
        scan()
    }
    
    func metadataOutput(_ output: AVCaptureMetadataOutput, didOutput metadataObjects: [AVMetadataObject], from connection: AVCaptureConnection) {
        captureSession.stopRunning()
        
        if let metadataObject = metadataObjects.first {
            guard let readableObject = metadataObject as? AVMetadataMachineReadableCodeObject else { return }
            guard let stringValue = readableObject.stringValue else { return }
            AudioServicesPlaySystemSound(SystemSoundID(kSystemSoundID_Vibrate))
            addrText.text = stringValue
            scanButton.isEnabled = true
        }
    }
    
    func scan(){
        captureSession = AVCaptureSession()
        
        guard let videoCaptureDevice = AVCaptureDevice.default(for: .video) else { return }
        let videoInput: AVCaptureDeviceInput
        
        do {
            videoInput = try AVCaptureDeviceInput(device: videoCaptureDevice)
        } catch {
            return
        }
        
        if (captureSession.canAddInput(videoInput)) {
            captureSession.addInput(videoInput)
        } else {
            failed()
            return
        }
        
        let metadataOutput = AVCaptureMetadataOutput()
        
        if (captureSession.canAddOutput(metadataOutput)) {
            captureSession.addOutput(metadataOutput)
            metadataOutput.setMetadataObjectsDelegate(self, queue: DispatchQueue.main)
            metadataOutput.metadataObjectTypes = [.qr]
        } else {
            failed()
            return
        }
        
        previewLayer = AVCaptureVideoPreviewLayer(session: captureSession)
        previewLayer.frame = qrView.layer.bounds
        previewLayer.videoGravity = .resizeAspectFill
        qrView.layer.addSublayer(previewLayer)
        captureSession.startRunning()
    }
    
    func failed() {
        let ac = UIAlertController(title: "Scanning not supported", message: "Your device does not support scanning. You can still input server address and port info manually and start sync.", preferredStyle: .alert)
        ac.addAction(UIAlertAction(title: "OK", style: .default))
        present(ac, animated: true)
        captureSession = nil
    }
    
    override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
        return .portrait
    }
    
}
