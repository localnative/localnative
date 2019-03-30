/*
 Local Native
 Copyright (C) 2018-2019  Yi Wang
 
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
//  QRCodeViewController.swift
//  localnative-ios
//
//  Created by Yi Wang on 3/30/19.
//

import UIKit
class QRCodeViewController : UIViewController{
    @IBOutlet weak var backButton: UIBarButtonItem!
    @IBOutlet weak var qrImage: UIImageView!
   
    @IBAction func backButtonTouchDown(_ sender: Any) {
        self.dismiss(animated: true, completion: nil)
    }
    
    func createQRFromString(_ str: String) {
        let size = qrImage.frame.size
        let stringData = str.data(using: .utf8)
        
        let qrFilter = CIFilter(name: "CIQRCodeGenerator")!
        qrFilter.setValue(stringData, forKey: "inputMessage")
        qrFilter.setValue("H", forKey: "inputCorrectionLevel")
        
        let minimalQRimage = qrFilter.outputImage!
        // NOTE that a QR code is always square, so minimalQRimage..width === .height
        let minimalSideLength = minimalQRimage.extent.width
        
        let smallestOutputExtent = (size.width < size.height) ? size.width : size.height
        let scaleFactor = smallestOutputExtent / minimalSideLength
        let scaledImage = minimalQRimage.transformed(
            by: CGAffineTransform(scaleX: scaleFactor, y: scaleFactor))
        
        qrImage.image = UIImage(ciImage: scaledImage,
                       scale: UIScreen.main.scale,
                       orientation: .up)
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()

    }
    
}
