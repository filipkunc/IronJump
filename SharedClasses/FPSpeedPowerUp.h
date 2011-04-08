//
//  FPSpeedPowerUp.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 7/17/10.
//  For license see LICENSE.TXT
//

#import "FPGameProtocols.h"

@interface FPSpeedPowerUp : NSObject <FPGameObject>
{
	float x, y;
	int speedUpCounter;
	BOOL isVisible;
}

@end

