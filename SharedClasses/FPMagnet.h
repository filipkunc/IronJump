//
//  FPMagnet.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 7/4/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPGameProtocols.h"

@interface FPMagnet : NSObject <FPGameObject> 
{
	float x, y;
	int widthSegments;	
	BOOL isVisible;
}

- (id)initWithWidthSegments:(int)aWidthSegments;

@end
